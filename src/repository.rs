//! Acesso ao Git pelo `git2`: o único módulo que fala com a libgit2.

use std::path::Path;

use chrono::{DateTime, Utc};
use git2::{BranchType, Delta, DiffFile, DiffFindOptions, ErrorCode, ObjectType, Oid, Sort};
use tempfile::TempDir;

use crate::error::{GitScopeError, Result};
use crate::model::{ChangeKind, CommitChanges, CommitRecord, FileChange, Filters};
use crate::remote;

pub struct Repository {
    // Declarado primeiro para o handle da libgit2 fechar antes de o diretório ser
    // removido (no Windows não dá para apagar arquivos abertos).
    inner: git2::Repository,
    temp: Option<TempDir>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloneProgress {
    pub received_objects: usize,
    pub total_objects: usize,
    pub received_bytes: usize,
}

impl Repository {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(GitScopeError::PathNotFound(path.to_path_buf()));
        }
        if !path.is_dir() {
            return Err(GitScopeError::NotARepository(path.to_path_buf()));
        }
        match git2::Repository::open(path) {
            Ok(inner) => Ok(Self { inner, temp: None }),
            Err(err) if err.code() == ErrorCode::NotFound => {
                Err(GitScopeError::NotARepository(path.to_path_buf()))
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn clone_to_temp(url: &str, mut on_progress: impl FnMut(CloneProgress)) -> Result<Self> {
        if remote::classify(url) != remote::UrlKind::Supported {
            return Err(GitScopeError::UnsupportedUrl {
                url: url.to_owned(),
                reason: "only https://, http://, git:// and file:// URLs are supported (SSH needs authentication, which GitScope does not handle)",
            });
        }

        let temp = tempfile::Builder::new().prefix("gitscope-").tempdir()?;
        let target = temp
            .path()
            .join(format!("{}.git", remote::repository_name(url)));

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.transfer_progress(|stats| {
            on_progress(CloneProgress {
                received_objects: stats.received_objects(),
                total_objects: stats.total_objects(),
                received_bytes: stats.received_bytes(),
            });
            true
        });
        let mut fetch = git2::FetchOptions::new();
        fetch.remote_callbacks(callbacks);

        let inner = git2::build::RepoBuilder::new()
            .bare(true)
            .fetch_options(fetch)
            .clone(url, &target)
            .map_err(|cause| GitScopeError::CloneFailed {
                url: url.to_owned(),
                cause,
            })?;
        Ok(Self {
            inner,
            temp: Some(temp),
        })
    }

    pub fn close(self) -> Result<()> {
        let Self { inner, temp } = self;
        drop(inner);
        if let Some(temp) = temp {
            temp.close()?;
        }
        Ok(())
    }

    pub fn is_temporary(&self) -> bool {
        self.temp.is_some()
    }

    pub fn workdir(&self) -> Option<&Path> {
        self.inner.workdir()
    }

    pub fn location(&self) -> &Path {
        self.inner.path()
    }

    pub fn name(&self) -> String {
        let base = self.inner.workdir().unwrap_or_else(|| self.inner.path());
        let name = base
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        name.strip_suffix(".git").unwrap_or(&name).to_owned()
    }

    pub fn branch_count(&self) -> Result<usize> {
        let kind = if self.is_temporary() {
            BranchType::Remote
        } else {
            BranchType::Local
        };
        let mut count = 0;
        for branch in self.inner.branches(Some(kind))? {
            let (branch, _) = branch?;
            // `origin/HEAD` é um alias, não uma branch própria.
            let is_alias = branch.name_bytes()?.ends_with(b"/HEAD");
            if !(kind == BranchType::Remote && is_alias) {
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn tag_count(&self) -> Result<usize> {
        Ok(self.inner.tag_names(None)?.len())
    }

    fn tip(&self, branch: Option<&str>) -> Result<Option<Oid>> {
        let Some(name) = branch else {
            return match self.inner.head() {
                Ok(head) => Ok(Some(head.peel_to_commit()?.id())),
                Err(err) if matches!(err.code(), ErrorCode::UnbornBranch | ErrorCode::NotFound) => {
                    Ok(None)
                }
                Err(err) => Err(err.into()),
            };
        };

        // Primeiro as branches locais, depois as remotas (`origin/main`): num clone
        // temporário, `--branch develop` só existe como `origin/develop`.
        let mut candidates = vec![
            (BranchType::Local, name.to_owned()),
            (BranchType::Remote, name.to_owned()),
        ];
        if self.is_temporary() {
            candidates.push((BranchType::Remote, format!("origin/{name}")));
        }
        for (kind, candidate) in candidates {
            match self.inner.find_branch(&candidate, kind) {
                Ok(found) => return Ok(Some(found.get().peel_to_commit()?.id())),
                Err(err) if matches!(err.code(), ErrorCode::NotFound | ErrorCode::InvalidSpec) => {}
                Err(err) => return Err(err.into()),
            }
        }
        Err(GitScopeError::BranchNotFound(name.to_owned()))
    }

    pub fn commits(&self, filters: &Filters) -> Result<Commits<'_>> {
        let walk = match self.tip(filters.branch.as_deref())? {
            Some(tip) => {
                let mut walk = self.inner.revwalk()?;
                walk.set_sorting(Sort::TIME)?;
                walk.push(tip)?;
                Some(walk)
            }
            None => None,
        };
        Ok(Commits {
            repo: &self.inner,
            walk,
            filters: filters.clone(),
        })
    }

    pub fn snapshot_files(&self, filters: &Filters) -> Result<Vec<String>> {
        let Some(tip) = self.tip(filters.branch.as_deref())? else {
            return Ok(Vec::new());
        };
        let tree = self.inner.find_commit(tip)?.tree()?;
        let mut paths = Vec::new();
        tree.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
            if entry.kind() == Some(ObjectType::Blob) {
                let name = String::from_utf8_lossy(entry.name_bytes());
                paths.push(format!("{dir}{name}"));
            }
            git2::TreeWalkResult::Ok
        })?;
        Ok(paths)
    }
}

pub struct Commits<'repo> {
    repo: &'repo git2::Repository,
    walk: Option<git2::Revwalk<'repo>>,
    filters: Filters,
}

impl Iterator for Commits<'_> {
    type Item = Result<CommitRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let oid = match self.walk.as_mut()?.next()? {
                Ok(oid) => oid,
                Err(err) => return Some(Err(err.into())),
            };
            match self.record(oid) {
                Ok(Some(record)) => return Some(Ok(record)),
                Ok(None) => continue,
                Err(err) => return Some(Err(err)),
            }
        }
    }
}

impl Commits<'_> {
    fn record(&self, oid: Oid) -> Result<Option<CommitRecord>> {
        let commit = self.repo.find_commit(oid)?;
        let author = commit.author();
        let name = String::from_utf8_lossy(author.name_bytes()).into_owned();
        let email = String::from_utf8_lossy(author.email_bytes()).into_owned();
        let date = DateTime::<Utc>::from_timestamp(author.when().seconds(), 0)
            .unwrap_or(DateTime::UNIX_EPOCH);

        if !self.filters.matches_date(date) || !self.filters.matches_author(&name, &email) {
            return Ok(None);
        }

        // Merges são contados, mas não diffados: as mudanças deles já estão nos commits
        // da branch mesclada.
        let is_merge = commit.parent_count() > 1;
        let changes = if is_merge {
            CommitChanges::default()
        } else {
            self.changes(&commit)?
        };

        Ok(Some(CommitRecord {
            id: oid.to_string(),
            author: name,
            email,
            date,
            message: String::from_utf8_lossy(commit.summary_bytes().unwrap_or_default())
                .into_owned(),
            is_merge,
            changes,
        }))
    }

    fn changes(&self, commit: &git2::Commit<'_>) -> Result<CommitChanges> {
        let tree = commit.tree()?;
        let parent_tree = if commit.parent_count() == 0 {
            None
        } else {
            Some(commit.parent(0)?.tree()?)
        };

        let mut diff = self
            .repo
            .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
        diff.find_similar(Some(DiffFindOptions::new().renames(true)))?;
        let stats = diff.stats()?;

        let mut files: Vec<FileChange> = diff
            .deltas()
            .filter_map(|delta| {
                let (kind, primary) = match delta.status() {
                    Delta::Added | Delta::Copied => (ChangeKind::Added, delta.new_file()),
                    Delta::Deleted => (ChangeKind::Deleted, delta.old_file()),
                    Delta::Modified | Delta::Typechange => (ChangeKind::Modified, delta.new_file()),
                    Delta::Renamed => (ChangeKind::Renamed, delta.new_file()),
                    _ => return None,
                };
                Some(FileChange {
                    path: path_of(&primary),
                    old_path: (kind == ChangeKind::Renamed).then(|| path_of(&delta.old_file())),
                    kind,
                })
            })
            .collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(CommitChanges {
            files,
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        })
    }
}

fn path_of(file: &DiffFile<'_>) -> String {
    file.path_bytes()
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .unwrap_or_default()
}
