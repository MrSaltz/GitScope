pub mod activity;
pub mod commits;
pub mod contributors;
pub mod files;
pub mod languages;

use std::cmp::Reverse;

use crate::error::Result;
use crate::model::{
    CommitInfo, CommitRecord, Filters, RepositoryInfo, RepositoryStats, SCHEMA_VERSION,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Options {
    pub commit_details: bool,
}

pub fn analyze(
    repository: RepositoryInfo,
    filters: Filters,
    commits: impl IntoIterator<Item = Result<CommitRecord>>,
    snapshot: impl IntoIterator<Item = String>,
    options: Options,
) -> Result<RepositoryStats> {
    let mut commit_acc = commits::CommitsAccumulator::default();
    let mut contributor_acc = contributors::ContributorsAccumulator::default();
    let mut file_acc = files::FilesAccumulator::default();
    let mut activity_acc = activity::ActivityAccumulator::default();
    let mut details: Vec<CommitInfo> = Vec::new();

    for record in commits {
        let record = record?;
        commit_acc.add(&record);
        contributor_acc.add(&record);
        file_acc.add(&record);
        activity_acc.add(&record);
        if options.commit_details {
            details.push(record.into());
        }
    }

    let snapshot: Vec<String> = snapshot.into_iter().collect();
    let languages = languages::language_stats(&snapshot);
    let total = commit_acc.total();

    // Mais novo primeiro; a ordenação é estável, então datas iguais mantêm a ordem
    // do percurso.
    details.sort_by_key(|c| Reverse(c.date));

    Ok(RepositoryStats {
        schema_version: SCHEMA_VERSION,
        repository,
        filters,
        commits: commit_acc.finish(),
        contributors: contributor_acc.finish(total),
        files: file_acc.finish(&snapshot),
        languages,
        activity: activity_acc.finish(),
        commit_details: options.commit_details.then_some(details),
    })
}

pub(crate) fn percentage(part: usize, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let value = part as f64 * 100.0 / total as f64;
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
pub(crate) mod testutil {
    use chrono::NaiveDateTime;

    use crate::model::{ChangeKind, CommitChanges, CommitRecord, FileChange};

    pub fn record(author: &str, email: &str, when: &str) -> CommitRecord {
        CommitRecord {
            id: format!("{author}-{when}"),
            author: author.to_owned(),
            email: email.to_owned(),
            date: NaiveDateTime::parse_from_str(when, "%Y-%m-%d %H:%M")
                .unwrap()
                .and_utc(),
            message: "msg".to_owned(),
            is_merge: false,
            changes: CommitChanges::default(),
        }
    }

    pub fn record_with(
        author: &str,
        email: &str,
        when: &str,
        paths: &[&str],
        insertions: usize,
        deletions: usize,
    ) -> CommitRecord {
        let mut r = record(author, email, when);
        r.changes = CommitChanges {
            files: paths
                .iter()
                .map(|p| FileChange {
                    path: (*p).to_owned(),
                    old_path: None,
                    kind: ChangeKind::Modified,
                })
                .collect(),
            insertions,
            deletions,
        };
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentage_is_rounded_and_safe_on_zero() {
        assert_eq!(percentage(1, 3), 33.33);
        assert_eq!(percentage(2, 3), 66.67);
        assert_eq!(percentage(1, 1), 100.0);
        assert_eq!(percentage(0, 0), 0.0);
    }
}
