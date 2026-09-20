mod common;

use chrono::Weekday;
use common::TestRepo;
use gitscope::analysis::Options;
use gitscope::model::{ChangeKind, Filters, RepositoryStats};
use gitscope::{GitScopeError, analyze_path};

const ALICE: (&str, &str) = ("Alice", "alice@example.com");
const BOB: (&str, &str) = ("Bob", "bob@example.com");
const CAROL: (&str, &str) = ("Carol", "carol@example.com");

fn run(t: &TestRepo, filters: Filters) -> RepositoryStats {
    analyze_path(t.path(), filters, Options::default()).unwrap()
}

fn run_detailed(t: &TestRepo, filters: Filters) -> RepositoryStats {
    let options = Options {
        commit_details: true,
    };
    analyze_path(t.path(), filters, options).unwrap()
}

fn date(s: &str) -> chrono::NaiveDate {
    Filters::parse_date(s).unwrap()
}

#[test]
fn empty_repository_yields_empty_statistics() {
    let t = TestRepo::named("empty");
    let stats = run(&t, Filters::default());

    assert_eq!(stats.repository.name, "empty");
    assert_eq!(stats.repository.branches, 0);
    assert_eq!(stats.commits.total, 0);
    assert_eq!(stats.commits.first, None);
    assert_eq!(stats.commits.most_active_hour, None);
    assert!(stats.contributors.is_empty());
    assert_eq!(stats.files.total, 0);
    assert!(stats.languages.is_empty());
    assert!(stats.activity.by_month.is_empty());
    assert!(stats.commit_details.is_none());
}

#[test]
fn single_commit() {
    let t = TestRepo::named("one");
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-03 12:30")
        .write("src/main.rs", "fn main() {}\n")
        .commit();

    let stats = run(&t, Filters::default());
    assert_eq!(stats.commits.total, 1);
    assert_eq!(stats.commits.first, stats.commits.last);
    assert_eq!(
        stats.commits.first.unwrap().to_rfc3339(),
        "2026-01-03T12:30:00+00:00"
    );
    assert_eq!(stats.contributors.len(), 1);
    assert_eq!(stats.contributors[0].percentage, 100.0);
    assert_eq!(stats.contributors[0].files_changed, 1);
    assert_eq!(stats.contributors[0].insertions, 1);
    assert_eq!(stats.files.total, 1);
    assert_eq!(stats.repository.branches, 1);
}

#[test]
fn multiple_commits_by_multiple_authors() {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-01 10:00")
        .write("a", "1")
        .commit();
    t.by(BOB.0, BOB.1)
        .at("2026-01-02 10:00")
        .write("b", "1")
        .commit();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-03 10:00")
        .write("c", "1")
        .commit();

    let stats = run(&t, Filters::default());
    assert_eq!(stats.commits.total, 3);
    let names: Vec<_> = stats
        .contributors
        .iter()
        .map(|c| (c.name.as_str(), c.commits, c.percentage))
        .collect();
    assert_eq!(names, vec![("Alice", 2, 66.67), ("Bob", 1, 33.33)]);
}

#[test]
fn commits_in_different_months_and_hours() {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-05 14:10")
        .write("a", "1")
        .commit();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-06 14:50")
        .write("a", "2")
        .commit();
    t.by(BOB.0, BOB.1)
        .at("2026-03-07 09:00")
        .write("b", "1")
        .commit();

    let stats = run(&t, Filters::default());
    assert_eq!(stats.commits.most_active_hour, Some(14));
    assert_eq!(stats.commits.by_hour[14].commits, 2);
    assert_eq!(stats.commits.by_hour[9].commits, 1);
    assert_eq!(stats.commits.by_weekday[0].weekday, Weekday::Mon);
    assert_eq!(stats.commits.by_weekday[0].commits, 1);
    assert_eq!(stats.commits.by_weekday[5].commits, 1);

    let months: Vec<_> = stats
        .activity
        .by_month
        .iter()
        .map(|m| (m.month.as_str(), m.commits))
        .collect();
    assert_eq!(months, vec![("2026-01", 2), ("2026-02", 0), ("2026-03", 1)]);
    assert_eq!(
        stats.commits.first.unwrap().to_rfc3339(),
        "2026-01-05T14:10:00+00:00"
    );
    assert_eq!(
        stats.commits.last.unwrap().to_rfc3339(),
        "2026-03-07T09:00:00+00:00"
    );
}

#[test]
fn timestamps_are_normalised_to_utc() {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-05 23:30")
        .offset(-180)
        .write("a", "1")
        .commit();

    let stats = run(&t, Filters::default());
    assert_eq!(stats.commits.most_active_hour, Some(23));
    assert_eq!(
        stats.commits.first.unwrap().to_rfc3339(),
        "2026-01-05T23:30:00+00:00"
    );
}

#[test]
fn since_and_until_are_inclusive_whole_days() {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1)
        .at("2026-05-31 23:59")
        .write("a", "1")
        .commit();
    t.by(ALICE.0, ALICE.1)
        .at("2026-06-01 00:00")
        .write("a", "2")
        .commit();
    t.by(ALICE.0, ALICE.1)
        .at("2026-06-30 23:59")
        .write("a", "3")
        .commit();
    t.by(ALICE.0, ALICE.1)
        .at("2026-07-01 00:00")
        .write("a", "4")
        .commit();

    let both = run(
        &t,
        Filters {
            since: Some(date("2026-06-01")),
            until: Some(date("2026-06-30")),
            ..Filters::default()
        },
    );
    assert_eq!(both.commits.total, 2);

    let since = run(
        &t,
        Filters {
            since: Some(date("2026-06-30")),
            ..Filters::default()
        },
    );
    assert_eq!(since.commits.total, 2);

    let until = run(
        &t,
        Filters {
            until: Some(date("2026-05-31")),
            ..Filters::default()
        },
    );
    assert_eq!(until.commits.total, 1);
}

#[test]
fn date_range_without_commits_is_empty_not_an_error() {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-01 10:00")
        .write("a", "1")
        .commit();

    let stats = run(
        &t,
        Filters {
            since: Some(date("2027-01-01")),
            ..Filters::default()
        },
    );
    assert_eq!(stats.commits.total, 0);
    assert!(stats.contributors.is_empty());
    assert_eq!(stats.files.total, 1);
}

#[test]
fn inverted_range_is_rejected() {
    let t = TestRepo::new();
    let err = analyze_path(
        t.path(),
        Filters {
            since: Some(date("2026-06-01")),
            until: Some(date("2026-01-01")),
            ..Filters::default()
        },
        Options::default(),
    )
    .unwrap_err();
    assert!(matches!(err, GitScopeError::InvalidRange { .. }));
}

fn three_author_repo() -> TestRepo {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-01 10:00")
        .write("a.rs", "1\n")
        .commit();
    t.by(BOB.0, BOB.1)
        .at("2026-01-02 10:00")
        .write("b.rs", "1\n2\n")
        .commit();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-03 10:00")
        .write("a.rs", "1\n2\n3\n")
        .commit();
    t
}

#[test]
fn author_filter_matches_name_case_insensitively() {
    let t = three_author_repo();
    let stats = run(
        &t,
        Filters {
            author: Some("aLiCe".into()),
            ..Filters::default()
        },
    );
    assert_eq!(stats.commits.total, 2);
    assert_eq!(stats.contributors.len(), 1);
    assert_eq!(stats.contributors[0].name, "Alice");
    assert_eq!(stats.contributors[0].percentage, 100.0);
}

#[test]
fn author_filter_matches_email() {
    let t = three_author_repo();
    let stats = run(
        &t,
        Filters {
            author: Some("bob@example".into()),
            ..Filters::default()
        },
    );
    assert_eq!(stats.commits.total, 1);
    assert_eq!(stats.contributors[0].name, "Bob");
}

#[test]
fn author_without_results_gives_empty_statistics() {
    let t = three_author_repo();
    let stats = run(
        &t,
        Filters {
            author: Some("Nobody".into()),
            ..Filters::default()
        },
    );
    assert_eq!(stats.commits.total, 0);
    assert!(stats.contributors.is_empty());
    assert!(stats.activity.by_month.is_empty());
    assert!(stats.files.most_modified.is_empty());
}

#[test]
fn files_extensions_and_languages_from_the_snapshot() {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1)
        .write("src/main.rs", "fn main() {}\n")
        .write("src/lib.rs", "\n")
        .write("web/app.ts", "\n")
        .write("tools/run.py", "\n")
        .write("README.md", "# hi\n")
        .write("Makefile", "all:\n")
        .commit();

    let stats = run(&t, Filters::default());
    assert_eq!(stats.files.total, 6);

    let ext: Vec<_> = stats
        .files
        .extensions
        .iter()
        .map(|e| (e.extension.as_deref(), e.count))
        .collect();
    assert_eq!(
        ext,
        vec![
            (Some("rs"), 2),
            (None, 1),
            (Some("md"), 1),
            (Some("py"), 1),
            (Some("ts"), 1),
        ]
    );

    let langs: Vec<_> = stats
        .languages
        .iter()
        .map(|l| (l.name.as_str(), l.files, l.percentage))
        .collect();
    assert_eq!(
        langs,
        vec![
            ("Rust", 2, 50.0),
            ("Python", 1, 25.0),
            ("TypeScript", 1, 25.0)
        ]
    );
}

#[test]
fn snapshot_reflects_deletions_and_renames() {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-01 10:00")
        .write("keep.rs", "1\n2\n3\n")
        .write("gone.py", "1\n")
        .write("old.ts", "1\n2\n3\n4\n")
        .commit();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-02 10:00")
        .remove("gone.py")
        .rename("old.ts", "new.ts")
        .commit();

    let stats = run(&t, Filters::default());
    assert_eq!(stats.files.total, 2);
    let names: Vec<_> = stats.languages.iter().map(|l| l.name.as_str()).collect();
    assert_eq!(names, vec!["Rust", "TypeScript"]);
}

#[test]
fn most_modified_counts_commits_per_file() {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-01 10:00")
        .write("a.rs", "1\n")
        .write("b.rs", "1\n")
        .commit();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-02 10:00")
        .write("a.rs", "2\n")
        .commit();
    t.by(BOB.0, BOB.1)
        .at("2026-01-03 10:00")
        .write("a.rs", "3\n")
        .write("c.rs", "1\n")
        .commit();

    let stats = run(&t, Filters::default());
    let ranking: Vec<_> = stats
        .files
        .most_modified
        .iter()
        .map(|f| (f.path.as_str(), f.modifications))
        .collect();
    assert_eq!(ranking, vec![("a.rs", 3), ("b.rs", 1), ("c.rs", 1)]);

    let bob = run(
        &t,
        Filters {
            author: Some("Bob".into()),
            ..Filters::default()
        },
    );
    let ranking: Vec<_> = bob
        .files
        .most_modified
        .iter()
        .map(|f| (f.path.as_str(), f.modifications))
        .collect();
    assert_eq!(ranking, vec![("a.rs", 1), ("c.rs", 1)]);
}

#[test]
fn counts_insertions_and_deletions() {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-01 10:00")
        .write("a.txt", "one\ntwo\nthree\n")
        .commit();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-02 10:00")
        .write("a.txt", "one\nTWO\nthree\nfour\n")
        .commit();

    let stats = run(&t, Filters::default());
    let alice = &stats.contributors[0];
    assert_eq!(alice.commits, 2);
    assert_eq!(alice.files_changed, 2);
    assert_eq!(alice.insertions, 3 + 2);
    assert_eq!(alice.deletions, 1);
}

#[test]
fn creation_modification_deletion_and_rename_are_classified() {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-01 10:00")
        .message("initial")
        .write("src/main.rs", "fn main() {}\n")
        .write("old_stats.rs", "// stats\n// more\n// lines\n")
        .write("notes.txt", "a\nb\n")
        .commit();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-02 10:00")
        .message("rework")
        .write("src/main.rs", "fn main() { run(); }\n")
        .write("src/output.rs", "// out\n")
        .remove("old_stats.rs")
        .rename("notes.txt", "docs/notes.txt")
        .commit();

    let stats = run_detailed(&t, Filters::default());
    let details = stats.commit_details.unwrap();
    assert_eq!(details.len(), 2);

    let rework = &details[0];
    assert_eq!(rework.message, "rework");
    let changes: Vec<_> = rework
        .changes
        .iter()
        .map(|c| (c.path.as_str(), c.old_path.as_deref(), c.kind))
        .collect();
    assert_eq!(
        changes,
        vec![
            ("docs/notes.txt", Some("notes.txt"), ChangeKind::Renamed),
            ("old_stats.rs", None, ChangeKind::Deleted),
            ("src/main.rs", None, ChangeKind::Modified),
            ("src/output.rs", None, ChangeKind::Added),
        ]
    );
    assert_eq!(rework.files_changed, 4);
    assert_eq!(rework.insertions, 2);
    assert_eq!(rework.deletions, 1 + 3);

    let initial = &details[1];
    assert_eq!(initial.files_changed, 3);
    assert!(initial.changes.iter().all(|c| c.kind == ChangeKind::Added));
    assert_eq!(initial.insertions, 1 + 3 + 2);
    assert_eq!(initial.deletions, 0);
}

#[test]
fn binary_files_count_as_changed_without_lines() {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1)
        .write("logo.png", "\u{0}\u{0}\u{0}binary\u{0}")
        .commit();

    let stats = run(&t, Filters::default());
    assert_eq!(stats.contributors[0].files_changed, 1);
    assert_eq!(stats.contributors[0].insertions, 0);
    assert_eq!(stats.contributors[0].deletions, 0);
}

fn merged_repo() -> TestRepo {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-01 10:00")
        .write("a.txt", "a\n")
        .commit();
    t.create_branch("feature");
    t.checkout("feature");
    t.by(BOB.0, BOB.1)
        .at("2026-01-02 10:00")
        .write("b.txt", "b\n")
        .commit();
    t.checkout("main");
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-03 10:00")
        .write("c.txt", "c\n")
        .commit();
    t.merge(
        "feature",
        CAROL.0,
        CAROL.1,
        "2026-01-04 10:00",
        "Merge feature",
    );
    t
}

#[test]
fn merge_commit_is_counted_but_not_diffed() {
    let t = merged_repo();
    let stats = run_detailed(&t, Filters::default());

    assert_eq!(stats.commits.total, 4);
    let carol = stats
        .contributors
        .iter()
        .find(|c| c.name == "Carol")
        .unwrap();
    assert_eq!(carol.commits, 1);
    assert_eq!(carol.files_changed, 0);
    assert_eq!(carol.insertions, 0);

    let bob = stats.contributors.iter().find(|c| c.name == "Bob").unwrap();
    assert_eq!(bob.files_changed, 1);

    let merge = &stats.commit_details.unwrap()[0];
    assert_eq!(merge.message, "Merge feature");
    assert!(merge.changes.is_empty());
    assert_eq!(stats.files.total, 3);
}

#[test]
fn branch_filter_limits_history_and_snapshot() {
    let t = merged_repo();

    let feature = run(
        &t,
        Filters {
            branch: Some("feature".into()),
            ..Filters::default()
        },
    );
    assert_eq!(feature.commits.total, 2);
    assert_eq!(feature.files.total, 2);
    assert!(feature.contributors.iter().all(|c| c.name != "Carol"));
    assert_eq!(feature.repository.branches, 2);
}

#[test]
fn default_history_is_head_and_ignores_unmerged_branches() {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-01 10:00")
        .write("a", "1")
        .commit();
    t.create_branch("wip");
    t.checkout("wip");
    t.by(BOB.0, BOB.1)
        .at("2026-01-02 10:00")
        .write("b", "1")
        .commit();
    t.checkout("main");

    let stats = run(&t, Filters::default());
    assert_eq!(stats.commits.total, 1);
    assert_eq!(stats.contributors.len(), 1);
}

#[test]
fn unknown_branch_is_an_error() {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1).write("a", "1").commit();

    let err = analyze_path(
        t.path(),
        Filters {
            branch: Some("nope".into()),
            ..Filters::default()
        },
        Options::default(),
    )
    .unwrap_err();
    assert!(matches!(err, GitScopeError::BranchNotFound(name) if name == "nope"));
}

#[test]
fn filters_combine() {
    let t = merged_repo();
    let stats = run(
        &t,
        Filters {
            author: Some("alice".into()),
            since: Some(date("2026-01-02")),
            branch: Some("main".into()),
            ..Filters::default()
        },
    );
    assert_eq!(stats.commits.total, 1);
    assert_eq!(stats.contributors[0].name, "Alice");
}

#[test]
fn commit_details_are_only_collected_on_request() {
    let t = three_author_repo();
    assert!(run(&t, Filters::default()).commit_details.is_none());

    let stats = run_detailed(
        &t,
        Filters {
            author: Some("Alice".into()),
            ..Filters::default()
        },
    );
    let details = stats.commit_details.unwrap();
    assert_eq!(details.len(), 2);
    assert!(details.iter().all(|c| c.author == "Alice"));
    assert_eq!(details[0].hash.len(), 40);
    assert_eq!(details[0].email, "alice@example.com");
    assert!(details[0].date > details[1].date);
}

#[test]
fn detached_head_is_analysed_from_where_it_points() {
    let t = TestRepo::new();
    let first = t
        .by(ALICE.0, ALICE.1)
        .at("2026-01-01 10:00")
        .write("a", "1")
        .commit();
    t.by(BOB.0, BOB.1)
        .at("2026-01-02 10:00")
        .write("b", "1")
        .commit();
    t.repo.set_head_detached(first).unwrap();

    let stats = run(&t, Filters::default());
    assert_eq!(stats.commits.total, 1);
    assert_eq!(stats.contributors[0].name, "Alice");
}

#[test]
fn remote_tracking_branches_can_be_selected() {
    let t = TestRepo::new();
    let first = t
        .by(ALICE.0, ALICE.1)
        .at("2026-01-01 10:00")
        .write("a", "1")
        .commit();
    t.by(BOB.0, BOB.1)
        .at("2026-01-02 10:00")
        .write("b", "1")
        .commit();
    t.repo
        .reference("refs/remotes/origin/release", first, true, "test")
        .unwrap();

    let stats = run(
        &t,
        Filters {
            branch: Some("origin/release".into()),
            ..Filters::default()
        },
    );
    assert_eq!(stats.commits.total, 1);
    assert_eq!(stats.repository.branches, 1);
}

#[test]
fn non_ascii_names_and_paths_survive() {
    let t = TestRepo::new();
    t.by("José Álvarez", "jose@example.com")
        .at("2026-01-01 10:00")
        .message("feat: acentuação")
        .write("docs/ação.md", "olá\n")
        .commit();

    let stats = run_detailed(&t, Filters::default());
    assert_eq!(stats.contributors[0].name, "José Álvarez");
    let details = stats.commit_details.unwrap();
    assert_eq!(details[0].message, "feat: acentuação");
    assert_eq!(details[0].changes[0].path, "docs/ação.md");

    let filtered = run(
        &t,
        Filters {
            author: Some("JOSÉ".into()),
            ..Filters::default()
        },
    );
    assert_eq!(filtered.commits.total, 1);
}

#[test]
fn a_commit_without_changes_is_still_a_commit() {
    let t = TestRepo::new();
    t.by(ALICE.0, ALICE.1)
        .at("2026-01-01 10:00")
        .write("a", "1")
        .commit();
    t.by(ALICE.0, ALICE.1).at("2026-01-02 10:00").commit();

    let stats = run_detailed(&t, Filters::default());
    assert_eq!(stats.commits.total, 2);
    assert_eq!(stats.contributors[0].files_changed, 1);
    let details = stats.commit_details.unwrap();
    assert_eq!(details[0].files_changed, 0);
    assert!(details[0].changes.is_empty());
}
