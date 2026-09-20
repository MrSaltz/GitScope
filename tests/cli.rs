mod common;

use assert_cmd::Command;
use common::TestRepo;
use predicates::prelude::*;
use serde_json::Value;

fn gitscope() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gitscope"))
}

fn sample_repo() -> TestRepo {
    let t = TestRepo::named("sample-project");
    t.by("Alice", "alice@example.com")
        .at("2026-01-05 14:10")
        .message("feat: add statistics")
        .write("src/main.rs", "fn main() {}\n")
        .write("README.md", "# sample\n")
        .commit();
    t.by("Bob", "bob@example.com")
        .at("2026-02-10 09:00")
        .message("fix: handle empty repositories")
        .write("src/main.rs", "fn main() { run(); }\n")
        .write("scripts/build.sh", "echo hi\n")
        .commit();
    t.by("Alice", "alice@example.com")
        .at("2026-03-07 14:50")
        .message("refactor: rename readme")
        .rename("README.md", "docs/README.md")
        .remove("scripts/build.sh")
        .commit();
    t.tag("v0.1.0");
    t
}

fn json_of(args: &[&str], repo: &TestRepo) -> Value {
    let output = gitscope()
        .arg(repo.path())
        .args(args)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .clone();
    assert!(
        output.stderr.is_empty(),
        "JSON mode must not write to stderr unless --verbose"
    );
    serde_json::from_slice(&output.stdout).expect("stdout must be valid JSON")
}

#[test]
fn missing_path_fails_with_a_clear_message() {
    let dir = tempfile::tempdir().unwrap();
    gitscope()
        .arg(dir.path().join("missing"))
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("error: path does not exist"));
}

#[test]
fn non_git_directory_fails() {
    let dir = tempfile::tempdir().unwrap();
    gitscope()
        .arg(dir.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not a Git repository"));
}

#[test]
fn unknown_branch_fails() {
    let t = sample_repo();
    gitscope()
        .arg(t.path())
        .args(["--branch", "nope"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("branch not found: 'nope'"));
}

#[test]
fn malformed_date_is_a_usage_error() {
    let t = sample_repo();
    gitscope()
        .arg(t.path())
        .args(["--since", "yesterday"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("expected the format YYYY-MM-DD"));
}

#[test]
fn inverted_range_fails() {
    let t = sample_repo();
    gitscope()
        .arg(t.path())
        .args(["--since", "2026-06-01", "--until", "2026-01-01"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "--since (2026-06-01) is after --until",
        ));
}

#[test]
fn overview_shows_every_section() {
    let t = sample_repo();
    gitscope()
        .arg(t.path())
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains("GITSCOPE v"))
        .stdout(predicate::str::contains("sample-project"))
        .stdout(predicate::str::is_match(r"Commits\s+3\n").unwrap())
        .stdout(predicate::str::is_match(r"Contributors\s+2\n").unwrap())
        .stdout(predicate::str::is_match(r"Tags\s+1\n").unwrap())
        .stdout(predicate::str::contains("Top contributors"))
        .stdout(predicate::str::is_match(r"Alice\s+2 commits\s+66\.7%").unwrap())
        .stdout(predicate::str::is_match(r"Bob\s+1 commit\s+33\.3%").unwrap())
        .stdout(predicate::str::is_match(r"First commit\s+2026-01-05").unwrap())
        .stdout(predicate::str::is_match(r"Last commit\s+2026-03-07").unwrap())
        .stdout(predicate::str::is_match(r"Most active hour\s+14:00").unwrap())
        .stdout(predicate::str::contains("Most modified files"))
        .stdout(predicate::str::is_match(r"src/main.rs\s+2\n").unwrap())
        .stdout(predicate::str::contains("Languages"))
        .stdout(predicate::str::contains("Rust"))
        .stdout(predicate::str::contains("Activity"))
        .stdout(predicate::str::contains("2026-02"));
}

#[test]
fn empty_repository_is_reported_without_error() {
    let t = TestRepo::named("fresh");
    gitscope()
        .arg(t.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "This repository has no commits yet.",
        ));
}

#[test]
fn author_without_results_is_reported_without_error() {
    let t = sample_repo();
    gitscope()
        .arg(t.path())
        .args(["--author", "Nobody"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No commits match the given filters.",
        ))
        .stdout(predicate::str::contains("author \"Nobody\""));
}

#[test]
fn author_view_shows_contributor_block() {
    let t = sample_repo();
    gitscope()
        .arg(t.path())
        .args(["--author", "alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Contributor"))
        .stdout(predicate::str::is_match(r"Name\s+Alice\n").unwrap())
        .stdout(predicate::str::is_match(r"Email\s+alice@example.com\n").unwrap())
        .stdout(predicate::str::is_match(r"Commits\s+2\n").unwrap())
        .stdout(predicate::str::is_match(r"Files changed\s+4\n").unwrap())
        .stdout(predicate::str::is_match(r"First commit\s+2026-01-05\n").unwrap())
        .stdout(predicate::str::is_match(r"Last commit\s+2026-03-07\n").unwrap())
        .stdout(predicate::str::contains("Bob").not())
        .stdout(predicate::str::contains("Commits by").not());
}

#[test]
fn all_lists_the_commits_of_the_author() {
    let t = sample_repo();
    gitscope()
        .arg(t.path())
        .args(["--author", "Alice", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Commits by Alice"))
        .stdout(
            predicate::str::is_match(r"2026-03-07  14:50  [0-9a-f]{7}\nrefactor: rename readme\n")
                .unwrap(),
        )
        .stdout(
            predicate::str::is_match(r"2026-01-05  14:10  [0-9a-f]{7}\nfeat: add statistics\n")
                .unwrap(),
        )
        .stdout(predicate::str::contains("fix: handle empty repositories").not())
        .stdout(predicate::str::contains("Changes\n").not());
}

#[test]
fn all_with_verbose_shows_full_commit_blocks() {
    let t = sample_repo();
    gitscope()
        .arg(t.path())
        .args(["--author", "Alice", "--all", "--verbose"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Author       Alice <alice@example.com>",
        ))
        .stdout(predicate::str::contains("Date         2026-03-07 14:50"))
        .stdout(predicate::str::contains(
            "Message      refactor: rename readme",
        ))
        .stdout(predicate::str::contains("  R README.md -> docs/README.md"))
        .stdout(predicate::str::contains("  - scripts/build.sh"))
        .stdout(predicate::str::contains("  + src/main.rs"))
        .stdout(predicate::str::is_match(r"Files changed\s+2\n").unwrap());
}

#[test]
fn top_limits_ranked_lists() {
    let t = sample_repo();
    gitscope()
        .arg(t.path())
        .args(["--top", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("… and 1 more"));
}

#[test]
fn verbose_writes_diagnostics_to_stderr_only() {
    let t = sample_repo();
    let plain = gitscope().arg(t.path()).output().unwrap();
    let verbose = gitscope().arg(t.path()).arg("--verbose").output().unwrap();

    let stderr = String::from_utf8(verbose.stderr).unwrap();
    assert!(
        stderr.contains("commits matched: 3"),
        "stderr was: {stderr}"
    );
    assert!(plain.stderr.is_empty());
    assert!(
        String::from_utf8(verbose.stdout)
            .unwrap()
            .contains("alice@example.com")
    );
}

#[test]
fn analyses_the_dot_git_directory_directly() {
    let t = sample_repo();
    gitscope()
        .arg(t.path().join(".git"))
        .assert()
        .success()
        .stdout(predicate::str::contains("sample-project"))
        .stdout(predicate::str::is_match(r"Commits\s+3\n").unwrap());
}

#[test]
fn date_filters_apply_to_the_output() {
    let t = sample_repo();
    gitscope()
        .arg(t.path())
        .args(["--since", "2026-02-01", "--until", "2026-02-28"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "since 2026-02-01 · until 2026-02-28",
        ))
        .stdout(predicate::str::is_match(r"Commits\s+1\n").unwrap())
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Alice").not());
}

#[test]
fn help_and_version() {
    gitscope()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--since"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--all"));
    gitscope()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

fn keys(value: &Value) -> Vec<&str> {
    value
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect()
}

#[test]
fn json_schema_is_stable() {
    let t = sample_repo();
    let json = json_of(&[], &t);

    // Qualquer mudança nestas listas de chaves muda o schema JSON público:
    // atualize docs/JSON.md e pense em `schema_version`.
    assert_eq!(
        keys(&json),
        [
            "activity",
            "commits",
            "contributors",
            "files",
            "filters",
            "languages",
            "repository",
            "schema_version"
        ]
    );
    assert_eq!(keys(&json["repository"]), ["branches", "name", "tags"]);
    assert_eq!(
        keys(&json["filters"]),
        ["author", "branch", "since", "until"]
    );
    assert_eq!(
        keys(&json["commits"]),
        [
            "by_hour",
            "by_weekday",
            "deletions",
            "first",
            "first_message",
            "insertions",
            "last",
            "last_message",
            "most_active_hour",
            "total"
        ]
    );
    assert_eq!(
        keys(&json["contributors"][0]),
        [
            "commits",
            "deletions",
            "email",
            "files_changed",
            "first_commit",
            "insertions",
            "last_commit",
            "name",
            "percentage"
        ]
    );
    assert_eq!(
        keys(&json["files"]),
        ["extensions", "most_modified", "total"]
    );
    assert_eq!(
        keys(&json["files"]["extensions"][0]),
        ["count", "extension"]
    );
    assert_eq!(
        keys(&json["files"]["most_modified"][0]),
        ["modifications", "path"]
    );
    assert_eq!(keys(&json["languages"][0]), ["files", "name", "percentage"]);
    assert_eq!(keys(&json["activity"]), ["by_month"]);
    assert_eq!(keys(&json["activity"]["by_month"][0]), ["commits", "month"]);
}

#[test]
fn json_carries_the_same_numbers_as_the_terminal() {
    let t = sample_repo();
    let json = json_of(&[], &t);

    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["repository"]["name"], "sample-project");
    assert_eq!(json["repository"]["branches"], 1);
    assert_eq!(json["repository"]["tags"], 1);
    assert_eq!(json["commits"]["total"], 3);
    assert_eq!(json["commits"]["first"], "2026-01-05T14:10:00Z");
    assert_eq!(json["commits"]["last"], "2026-03-07T14:50:00Z");
    assert_eq!(json["commits"]["most_active_hour"], 14);
    assert_eq!(json["commits"]["by_weekday"].as_array().unwrap().len(), 7);
    assert_eq!(json["commits"]["by_weekday"][0]["weekday"], "Mon");
    assert_eq!(json["commits"]["by_hour"].as_array().unwrap().len(), 24);

    let contributors = json["contributors"].as_array().unwrap();
    assert_eq!(contributors.len(), 2);
    assert_eq!(contributors[0]["name"], "Alice");
    assert_eq!(contributors[0]["commits"], 2);
    assert_eq!(contributors[0]["percentage"], 66.67);
    assert_eq!(contributors[0]["files_changed"], 4);
    assert_eq!(contributors[1]["name"], "Bob");

    let months = json["activity"]["by_month"].as_array().unwrap();
    assert_eq!(months.len(), 3);
    assert_eq!(months[0]["month"], "2026-01");

    assert_eq!(json["files"]["total"], 2);
    assert_eq!(json["languages"][0]["name"], "Rust");

    assert!(json.get("commit_details").is_none());
    assert!(json["filters"]["author"].is_null());
}

#[test]
fn json_with_author_and_all_includes_detailed_commits() {
    let t = sample_repo();
    let json = json_of(&["--author", "Alice", "--all"], &t);

    assert_eq!(json["filters"]["author"], "Alice");
    assert_eq!(json["contributors"].as_array().unwrap().len(), 1);

    let details = json["commit_details"].as_array().unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(
        keys(&details[0]),
        [
            "author",
            "changes",
            "date",
            "deletions",
            "email",
            "files_changed",
            "hash",
            "insertions",
            "message"
        ]
    );
    assert_eq!(details[0]["message"], "refactor: rename readme");
    assert_eq!(details[0]["date"], "2026-03-07T14:50:00Z");
    assert_eq!(details[0]["hash"].as_str().unwrap().len(), 40);
    assert_eq!(details[0]["files_changed"], 2);
    assert_eq!(details[0]["deletions"], 1);

    let changes = details[0]["changes"].as_array().unwrap();
    assert_eq!(changes[0]["kind"], "renamed");
    assert_eq!(changes[0]["path"], "docs/README.md");
    assert_eq!(changes[0]["old_path"], "README.md");
    assert_eq!(changes[1]["kind"], "deleted");
    assert!(changes[1].get("old_path").is_none());
    assert_eq!(details[1]["changes"][0]["kind"], "added");
}

#[test]
fn json_is_valid_for_empty_repositories_and_empty_results() {
    let empty = TestRepo::named("fresh");
    let json = json_of(&[], &empty);
    assert_eq!(json["commits"]["total"], 0);
    assert!(json["commits"]["first"].is_null());
    assert!(json["commits"]["most_active_hour"].is_null());
    assert_eq!(json["contributors"].as_array().unwrap().len(), 0);

    let t = sample_repo();
    let json = json_of(&["--author", "Nobody", "--all"], &t);
    assert_eq!(json["commits"]["total"], 0);
    assert_eq!(json["commit_details"].as_array().unwrap().len(), 0);
}

#[test]
fn json_is_not_truncated_by_top() {
    let t = sample_repo();
    let json = json_of(&["--top", "1"], &t);
    assert_eq!(json["contributors"].as_array().unwrap().len(), 2);
}

#[test]
fn json_stays_clean_with_verbose() {
    let t = sample_repo();
    let output = gitscope()
        .arg(t.path())
        .args(["--json", "--verbose"])
        .assert()
        .success()
        .get_output()
        .clone();
    serde_json::from_slice::<Value>(&output.stdout).expect("stdout must stay valid JSON");
    assert!(!output.stderr.is_empty());
}

#[test]
fn json_reflects_branch_and_date_filters() {
    let t = sample_repo();
    let json = json_of(
        &[
            "--branch",
            "main",
            "--since",
            "2026-02-01",
            "--until",
            "2026-02-28",
        ],
        &t,
    );
    assert_eq!(json["filters"]["branch"], "main");
    assert_eq!(json["filters"]["since"], "2026-02-01");
    assert_eq!(json["filters"]["until"], "2026-02-28");
    assert_eq!(json["commits"]["total"], 1);
}

#[test]
fn tui_refuses_to_run_without_a_terminal_and_says_what_to_use() {
    let t = sample_repo();
    // O assert_cmd liga stdin/stdout a pipes, então não há terminal.
    gitscope()
        .arg("tui")
        .arg(t.path())
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "the interactive interface needs a terminal",
        ))
        .stderr(predicate::str::contains("--json"));
}

#[test]
fn tui_is_listed_in_the_help_and_rejects_report_options() {
    gitscope()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("tui"))
        .stdout(predicate::str::contains("interactive"));
    gitscope()
        .args(["tui", "--json"])
        .assert()
        .failure()
        .code(2);
}
