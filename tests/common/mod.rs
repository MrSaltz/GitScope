//! Monta repositórios Git pequenos e determinísticos com o `git2` (nunca chama o `git` por shell).
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use git2::{BranchType, Oid, Repository, Signature, Time, build::CheckoutBuilder};
use tempfile::TempDir;

pub struct TestRepo {
    _dir: TempDir,
    path: PathBuf,
    pub repo: Repository,
}

enum Op {
    Write(String, String),
    Remove(String),
    Rename(String, String),
}

pub struct CommitBuilder<'a> {
    test: &'a TestRepo,
    name: String,
    email: String,
    when: i64,
    offset_minutes: i32,
    message: String,
    ops: Vec<Op>,
}

pub fn ts(when: &str) -> i64 {
    NaiveDateTime::parse_from_str(when, "%Y-%m-%d %H:%M")
        .unwrap_or_else(|e| panic!("bad test date '{when}': {e}"))
        .and_utc()
        .timestamp()
}

impl TestRepo {
    pub fn new() -> Self {
        Self::named("test-repo")
    }

    pub fn named(name: &str) -> Self {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join(name);
        fs::create_dir(&path).expect("create repo dir");
        let mut opts = git2::RepositoryInitOptions::new();
        opts.initial_head("main");
        let repo = Repository::init_opts(&path, &opts).expect("git init");
        Self {
            _dir: dir,
            path,
            repo,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn by(&self, name: &str, email: &str) -> CommitBuilder<'_> {
        CommitBuilder {
            test: self,
            name: name.to_owned(),
            email: email.to_owned(),
            when: ts("2026-01-01 00:00"),
            offset_minutes: 0,
            message: "commit".to_owned(),
            ops: Vec::new(),
        }
    }

    pub fn create_branch(&self, name: &str) {
        let head = self.repo.head().unwrap().peel_to_commit().unwrap();
        self.repo.branch(name, &head, false).unwrap();
    }

    pub fn checkout(&self, name: &str) {
        self.repo.set_head(&format!("refs/heads/{name}")).unwrap();
        self.repo
            .checkout_head(Some(CheckoutBuilder::new().force()))
            .unwrap();
    }

    pub fn tag(&self, name: &str) {
        let head = self.repo.head().unwrap().peel_to_commit().unwrap();
        self.repo
            .tag_lightweight(name, head.as_object(), false)
            .unwrap();
    }

    pub fn annotated_tag(&self, name: &str) {
        let head = self.repo.head().unwrap().peel_to_commit().unwrap();
        let sig = Signature::now("Tagger", "tagger@example.com").unwrap();
        self.repo
            .tag(name, head.as_object(), &sig, "annotated", false)
            .unwrap();
    }

    pub fn merge(&self, branch: &str, name: &str, email: &str, when: &str, message: &str) -> Oid {
        let our = self.repo.head().unwrap().peel_to_commit().unwrap();
        let their = self
            .repo
            .find_branch(branch, BranchType::Local)
            .unwrap()
            .get()
            .peel_to_commit()
            .unwrap();
        let mut index = self.repo.merge_commits(&our, &their, None).unwrap();
        assert!(!index.has_conflicts(), "test merge must not conflict");
        let tree = self
            .repo
            .find_tree(index.write_tree_to(&self.repo).unwrap())
            .unwrap();
        let sig = Signature::new(name, email, &Time::new(ts(when), 0)).unwrap();
        let oid = self
            .repo
            .commit(Some("HEAD"), &sig, &sig, message, &tree, &[&our, &their])
            .unwrap();
        self.repo
            .checkout_head(Some(CheckoutBuilder::new().force()))
            .unwrap();
        oid
    }
}

impl CommitBuilder<'_> {
    pub fn at(mut self, when: &str) -> Self {
        self.when = ts(when);
        self
    }

    pub fn offset(mut self, minutes: i32) -> Self {
        self.offset_minutes = minutes;
        self
    }

    pub fn message(mut self, message: &str) -> Self {
        self.message = message.to_owned();
        self
    }

    pub fn write(mut self, path: &str, content: &str) -> Self {
        self.ops.push(Op::Write(path.into(), content.into()));
        self
    }

    pub fn remove(mut self, path: &str) -> Self {
        self.ops.push(Op::Remove(path.into()));
        self
    }

    pub fn rename(mut self, from: &str, to: &str) -> Self {
        self.ops.push(Op::Rename(from.into(), to.into()));
        self
    }

    pub fn commit(self) -> Oid {
        let repo = &self.test.repo;
        let root = self.test.path();
        let mut index = repo.index().unwrap();

        for op in &self.ops {
            match op {
                Op::Write(path, content) => {
                    let full = root.join(path);
                    fs::create_dir_all(full.parent().unwrap()).unwrap();
                    fs::write(&full, content).unwrap();
                    index.add_path(Path::new(path)).unwrap();
                }
                Op::Remove(path) => {
                    fs::remove_file(root.join(path)).unwrap();
                    index.remove_path(Path::new(path)).unwrap();
                }
                Op::Rename(from, to) => {
                    let dest = root.join(to);
                    fs::create_dir_all(dest.parent().unwrap()).unwrap();
                    fs::rename(root.join(from), &dest).unwrap();
                    index.remove_path(Path::new(from)).unwrap();
                    index.add_path(Path::new(to)).unwrap();
                }
            }
        }
        index.write().unwrap();
        let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();

        let sig = Signature::new(
            &self.name,
            &self.email,
            &Time::new(self.when, self.offset_minutes),
        )
        .unwrap();
        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = parent.iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, &self.message, &tree, &parents)
            .unwrap()
    }
}
