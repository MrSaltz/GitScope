mod common;

use std::path::Path;

use assert_cmd::Command;
use common::TestRepo;
use gitscope::analysis::Options;
use gitscope::model::Filters;
use gitscope::{GitScopeError, analyze_repository, is_remote, open_source};
use predicates::prelude::*;
use serde_json::Value;

fn file_url(path: &Path) -> String {
    let path = path.to_str().unwrap().replace('\\', "/");
    if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    }
}

fn sample_repo() -> TestRepo {
    let t = TestRepo::named("sample-project");
    t.by("Alice", "alice@example.com")
        .at("2026-01-05 14:10")
        .message("feat: add statistics")
        .write("src/main.rs", "fn main() {}\n")
        .commit();
    t.by("Bob", "bob@example.com")
        .at("2026-02-10 09:00")
        .message("fix: handle empty repositories")
        .write("src/main.rs", "fn main() { run(); }\n")
        .write("web/app.ts", "export {};\n")
        .commit();
    t.tag("v0.1.0");
    t
}

fn gitscope_in(tmp: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_gitscope"));
    cmd.env("TMPDIR", tmp).env("TMP", tmp).env("TEMP", tmp);
    cmd
}

fn entries(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect()
}

fn json_output(cmd: &mut Command) -> Value {
    let output = cmd.arg("--json").assert().success().get_output().clone();
    serde_json::from_slice(&output.stdout).expect("stdout must be valid JSON")
}

#[test]
fn urls_are_remote_and_paths_are_not() {
    let t = sample_repo();
    assert!(!is_remote(t.path()), "an existing path is never remote");
    assert!(!is_remote("./does-not-exist"));
    assert!(!is_remote(r"C:\missing\repo"));
    assert!(is_remote("https://github.com/owner/repo.git"));
    assert!(is_remote(file_url(t.path())));
    assert!(is_remote("git@github.com:owner/repo.git"));
}

#[test]
fn a_local_directory_named_like_a_url_is_still_local() {
    let dir = tempfile::tempdir().unwrap();
    let odd = dir.path().join("https:");
    if std::fs::create_dir(&odd).is_err() {
        return;
    }
    let inside = odd.join("repo");
    std::fs::create_dir(&inside).unwrap();
    assert!(!is_remote(&inside));
}

#[test]
fn open_source_clones_a_url_and_close_deletes_the_clone() {
    let t = sample_repo();
    let repo = open_source(file_url(t.path()), |_| {}).unwrap();
    assert!(repo.is_temporary());
    let location = repo.location().to_path_buf();
    assert!(location.exists());

    let stats = analyze_repository(&repo, Filters::default(), Options::default()).unwrap();
    assert_eq!(stats.repository.name, "sample-project");
    assert_eq!(stats.commits.total, 2);

    repo.close().unwrap();
    assert!(!location.exists(), "temporary clone must be deleted");
}

#[test]
fn dropping_a_temporary_clone_deletes_it_too() {
    let t = sample_repo();
    let repo = open_source(file_url(t.path()), |_| {}).unwrap();
    let location = repo.location().to_path_buf();
    drop(repo);
    assert!(!location.exists());
}

#[test]
fn local_paths_are_opened_in_place() {
    let t = sample_repo();
    let repo = open_source(t.path(), |_| {}).unwrap();
    assert!(!repo.is_temporary());
    repo.close().unwrap();
    assert!(
        t.path().exists(),
        "closing a local repository must not delete it"
    );
}

#[test]
fn ssh_urls_are_rejected_with_an_explanation() {
    for url in [
        "ssh://git@github.com/owner/repo.git",
        "git@github.com:owner/repo.git",
    ] {
        let err = open_source(url, |_| {}).err().expect("must fail");
        assert!(
            matches!(&err, GitScopeError::UnsupportedUrl { url: u, .. } if u == url),
            "{err}"
        );
        assert!(err.to_string().contains("https://"));
    }
}

#[test]
fn cloning_an_empty_repository_works() {
    let t = TestRepo::named("empty-project");
    let repo = open_source(file_url(t.path()), |_| {}).unwrap();
    let stats = analyze_repository(&repo, Filters::default(), Options::default()).unwrap();
    assert_eq!(stats.commits.total, 0);
    assert_eq!(stats.repository.name, "empty-project");
}

#[test]
fn a_cloned_repository_gives_the_same_report_as_the_local_one() {
    let t = sample_repo();
    let tmp = tempfile::tempdir().unwrap();

    let local = json_output(gitscope_in(tmp.path()).arg(t.path()));
    let remote = json_output(gitscope_in(tmp.path()).arg(file_url(t.path())));

    assert_eq!(local, remote);
    assert_eq!(remote["repository"]["name"], "sample-project");
    assert_eq!(remote["repository"]["tags"], 1);
    assert_eq!(remote["commits"]["total"], 2);
}

#[test]
fn stdout_stays_clean_and_stderr_announces_the_clone() {
    let t = sample_repo();
    let tmp = tempfile::tempdir().unwrap();

    gitscope_in(tmp.path())
        .arg(file_url(t.path()))
        .arg("--json")
        .assert()
        .success()
        .stderr(predicate::str::contains("cloning"))
        .stdout(predicate::str::starts_with("{"));
}

#[test]
fn the_temporary_clone_is_removed_after_a_successful_run() {
    let t = sample_repo();
    let tmp = tempfile::tempdir().unwrap();

    gitscope_in(tmp.path())
        .arg(file_url(t.path()))
        .arg("--verbose")
        .assert()
        .success()
        .stderr(predicate::str::contains("temporary clone removed"));

    assert_eq!(entries(tmp.path()), Vec::<String>::new());
}

#[test]
fn the_temporary_clone_is_removed_when_the_analysis_fails() {
    let t = sample_repo();
    let tmp = tempfile::tempdir().unwrap();

    gitscope_in(tmp.path())
        .arg(file_url(t.path()))
        .args(["--branch", "does-not-exist"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "branch not found: 'does-not-exist'",
        ));

    assert_eq!(entries(tmp.path()), Vec::<String>::new());
}

#[test]
fn a_failed_clone_reports_the_url_and_leaves_nothing_behind() {
    let tmp = tempfile::tempdir().unwrap();
    // Nada escuta na porta 1, então falha na hora e sem rede.
    let url = "https://127.0.0.1:1/owner/repo.git";

    gitscope_in(tmp.path())
        .arg(url)
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(format!("failed to clone '{url}'")));

    assert_eq!(entries(tmp.path()), Vec::<String>::new());
}

#[test]
fn ssh_urls_fail_fast_without_cloning() {
    let tmp = tempfile::tempdir().unwrap();

    gitscope_in(tmp.path())
        .arg("git@github.com:owner/repo.git")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("unsupported repository URL"))
        .stderr(predicate::str::contains("https://"))
        .stderr(predicate::str::contains("cloning").not());

    assert_eq!(entries(tmp.path()), Vec::<String>::new());
}

#[test]
fn bad_filters_are_rejected_before_cloning() {
    let tmp = tempfile::tempdir().unwrap();

    gitscope_in(tmp.path())
        .arg("https://127.0.0.1:1/owner/repo.git")
        .args(["--since", "2026-06-01", "--until", "2026-01-01"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("is after --until"))
        .stderr(predicate::str::contains("cloning").not());
}

#[test]
fn every_branch_of_the_remote_is_counted_and_selectable() {
    let t = sample_repo();
    t.create_branch("feature");
    t.checkout("feature");
    t.by("Carol", "carol@example.com")
        .at("2026-03-01 10:00")
        .message("feat: wip")
        .write("wip.rs", "//\n")
        .commit();
    t.checkout("main");
    let tmp = tempfile::tempdir().unwrap();
    let url = file_url(t.path());

    let default = json_output(gitscope_in(tmp.path()).arg(&url));
    assert_eq!(default["repository"]["branches"], 2);
    assert_eq!(default["commits"]["total"], 2);

    let feature = json_output(
        gitscope_in(tmp.path())
            .arg(&url)
            .args(["--branch", "feature"]),
    );
    assert_eq!(feature["commits"]["total"], 3);
    assert_eq!(feature["contributors"].as_array().unwrap().len(), 3);

    let tracking = json_output(
        gitscope_in(tmp.path())
            .arg(&url)
            .args(["--branch", "origin/feature"]),
    );
    assert_eq!(tracking["commits"]["total"], 3);

    assert_eq!(entries(tmp.path()), Vec::<String>::new());
}

#[test]
fn filters_and_all_work_on_clones() {
    let t = sample_repo();
    let tmp = tempfile::tempdir().unwrap();

    let json = json_output(
        gitscope_in(tmp.path())
            .arg(file_url(t.path()))
            .args(["--author", "bob", "--all"]),
    );
    assert_eq!(json["commits"]["total"], 1);
    let details = json["commit_details"].as_array().unwrap();
    assert_eq!(details.len(), 1);
    assert_eq!(details[0]["message"], "fix: handle empty repositories");
    assert_eq!(details[0]["files_changed"], 2);
}

#[test]
fn credentials_in_a_url_are_never_printed() {
    let tmp = tempfile::tempdir().unwrap();

    let output = gitscope_in(tmp.path())
        .arg("https://alice:s3cret-token@127.0.0.1:1/owner/repo.git")
        .arg("--verbose")
        .assert()
        .failure()
        .code(1)
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("s3cret-token"), "leaked: {stderr}");
    assert!(!stderr.contains("alice"), "leaked: {stderr}");
    assert!(stderr.contains("https://***@127.0.0.1:1/owner/repo.git"));
    assert_eq!(entries(tmp.path()), Vec::<String>::new());
}
