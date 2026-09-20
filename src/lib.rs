//! GitScope: estatísticas de repositórios Git, locais ou remotos.

pub mod analysis;
pub mod cli;
pub mod config;
pub mod error;
pub mod model;
pub mod output;
pub mod remote;
pub mod repository;

use std::path::Path;

pub use error::{GitScopeError, Result};
pub use model::{Filters, RepositoryStats};

use model::RepositoryInfo;
use remote::UrlKind;
use repository::{CloneProgress, Repository};

pub fn is_remote(source: impl AsRef<Path>) -> bool {
    let source = source.as_ref();
    !source.exists()
        && source
            .to_str()
            .is_some_and(|url| remote::classify(url) != UrlKind::NotUrl)
}

pub fn open_source(
    source: impl AsRef<Path>,
    on_progress: impl FnMut(CloneProgress),
) -> Result<Repository> {
    let source = source.as_ref();
    match source.to_str() {
        Some(url) if is_remote(source) => Repository::clone_to_temp(url, on_progress),
        _ => Repository::open(source),
    }
}

pub fn analyze_repository(
    repo: &Repository,
    filters: Filters,
    options: analysis::Options,
) -> Result<RepositoryStats> {
    filters.validate()?;
    let info = RepositoryInfo {
        name: repo.name(),
        branches: repo.branch_count()?,
        tags: repo.tag_count()?,
    };
    let commits = repo.commits(&filters)?;
    let snapshot = repo.snapshot_files(&filters)?;
    analysis::analyze(info, filters.clone(), commits, snapshot, options)
}

pub fn analyze_path(
    path: impl AsRef<Path>,
    filters: Filters,
    options: analysis::Options,
) -> Result<RepositoryStats> {
    filters.validate()?;
    let repo = Repository::open(path)?;
    analyze_repository(&repo, filters, options)
}
