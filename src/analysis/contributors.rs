use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::percentage;
use crate::model::{CommitRecord, ContributorStats};

#[derive(Debug)]
struct Entry {
    name: String,
    email: String,
    identity_date: DateTime<Utc>,
    commits: usize,
    files_changed: usize,
    insertions: usize,
    deletions: usize,
    first: DateTime<Utc>,
    last: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct ContributorsAccumulator {
    entries: HashMap<String, Entry>,
}

impl ContributorsAccumulator {
    pub fn add(&mut self, commit: &CommitRecord) {
        let entry = self
            .entries
            .entry(commit.email.to_lowercase())
            .or_insert_with(|| Entry {
                name: commit.author.clone(),
                email: commit.email.clone(),
                identity_date: commit.date,
                commits: 0,
                files_changed: 0,
                insertions: 0,
                deletions: 0,
                first: commit.date,
                last: commit.date,
            });

        entry.commits += 1;
        entry.files_changed += commit.changes.files.len();
        entry.insertions += commit.changes.insertions;
        entry.deletions += commit.changes.deletions;
        entry.first = entry.first.min(commit.date);
        entry.last = entry.last.max(commit.date);
        if commit.date > entry.identity_date {
            entry.identity_date = commit.date;
            entry.name.clone_from(&commit.author);
            entry.email.clone_from(&commit.email);
        }
    }

    pub fn finish(self, total_commits: usize) -> Vec<ContributorStats> {
        let mut list: Vec<ContributorStats> = self
            .entries
            .into_values()
            .map(|e| ContributorStats {
                percentage: percentage(e.commits, total_commits),
                name: e.name,
                email: e.email,
                commits: e.commits,
                files_changed: e.files_changed,
                insertions: e.insertions,
                deletions: e.deletions,
                first_commit: e.first,
                last_commit: e.last,
            })
            .collect();
        list.sort_by(|a, b| {
            b.commits
                .cmp(&a.commits)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                .then_with(|| a.email.cmp(&b.email))
        });
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::testutil::{record, record_with};

    fn run(records: Vec<CommitRecord>) -> Vec<ContributorStats> {
        let total = records.len();
        let mut acc = ContributorsAccumulator::default();
        for r in &records {
            acc.add(r);
        }
        acc.finish(total)
    }

    #[test]
    fn alice_bob_alice_gives_alice_two_and_bob_one() {
        let list = run(vec![
            record("Alice", "alice@x.io", "2026-01-01 10:00"),
            record("Bob", "bob@x.io", "2026-01-02 10:00"),
            record("Alice", "alice@x.io", "2026-01-03 10:00"),
        ]);
        assert_eq!(list.len(), 2);
        assert_eq!((list[0].name.as_str(), list[0].commits), ("Alice", 2));
        assert_eq!((list[1].name.as_str(), list[1].commits), ("Bob", 1));
        assert_eq!(list[0].percentage, 66.67);
        assert_eq!(list[1].percentage, 33.33);
    }

    #[test]
    fn sums_files_and_lines_and_tracks_dates() {
        let list = run(vec![
            record_with(
                "Alice",
                "alice@x.io",
                "2026-01-01 10:00",
                &["a", "b"],
                10,
                1,
            ),
            record_with("Alice", "alice@x.io", "2026-02-01 10:00", &["a"], 5, 4),
        ]);
        let alice = &list[0];
        assert_eq!(alice.files_changed, 3);
        assert_eq!(alice.insertions, 15);
        assert_eq!(alice.deletions, 5);
        assert_eq!(alice.first_commit.to_rfc3339(), "2026-01-01T10:00:00+00:00");
        assert_eq!(alice.last_commit.to_rfc3339(), "2026-02-01T10:00:00+00:00");
    }

    #[test]
    fn same_email_with_different_names_is_one_contributor() {
        let list = run(vec![
            record("Bob", "Bob@X.io", "2026-01-01 10:00"),
            record("Bob Smith", "bob@x.io", "2026-03-01 10:00"),
        ]);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].commits, 2);
        assert_eq!(list[0].name, "Bob Smith");
        assert_eq!(list[0].email, "bob@x.io");
    }

    #[test]
    fn identity_does_not_depend_on_input_order() {
        let list = run(vec![
            record("Bob Smith", "bob@x.io", "2026-03-01 10:00"),
            record("Bob", "bob@x.io", "2026-01-01 10:00"),
        ]);
        assert_eq!(list[0].name, "Bob Smith");
    }

    #[test]
    fn ties_are_ordered_by_name() {
        let list = run(vec![
            record("Zoe", "zoe@x.io", "2026-01-01 10:00"),
            record("alice", "alice@x.io", "2026-01-02 10:00"),
        ]);
        assert_eq!(list[0].name, "alice");
        assert_eq!(list[1].name, "Zoe");
    }

    #[test]
    fn no_commits_no_contributors() {
        assert!(run(Vec::new()).is_empty());
    }
}
