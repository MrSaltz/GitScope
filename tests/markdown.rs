mod common;

use std::fs;
use std::path::Path;

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

const README: &str = "\
# Sample

Hand-written intro that must never change: {commits} stays here.

## Statistics

<!-- gitscope:start -->

Project started: {first_commit}

Latest commit: {last_commit}

Total commits: {commits}

Contributors: {contributors}

Files: {files}

Lines added: {insertions}

Lines deleted: {deletions}

<!-- gitscope:end -->

Footer, also untouched: {files}
";

fn write_readme(repo: &TestRepo, text: &str) -> std::path::PathBuf {
    let path = repo.path().join("README.md");
    fs::write(&path, text).unwrap();
    path
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

fn assert_no_temp_files(dir: &Path) {
    let leftovers: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(".gitscope-") || n.ends_with(".tmp"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "temporary files left behind: {leftovers:?}"
    );
}

#[test]
fn update_finds_the_readme_at_the_repository_root() {
    let t = sample_repo();
    let readme = write_readme(&t, README);

    gitscope()
        .arg("update")
        .current_dir(t.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated"))
        .stdout(predicate::str::contains("README.md (1 block, 7 variables)"));

    let updated = read(&readme);
    assert!(updated.starts_with("# Sample\n\nHand-written intro that must never change: {commits} stays here.\n\n## Statistics\n\n<!-- gitscope:start -->\n"));
    assert!(updated.ends_with("<!-- gitscope:end -->\n\nFooter, also untouched: {files}\n"));
    assert!(updated.contains("Project started: 2026-01-05\n"));
    assert!(updated.contains("Latest commit: 2026-03-07\n"));
    assert!(updated.contains("Total commits: 3\n"));
    assert!(updated.contains("Contributors: 2\n"));
    assert!(updated.contains("Files: 2\n"));
    assert!(updated.contains("Lines added: 4\n"));
    assert!(updated.contains("Lines deleted: 2\n"));
    assert!(updated.contains("<!-- gitscope:template\n"));
    assert!(updated.contains("Total commits: {commits}\n"));
}

#[test]
fn updating_again_changes_nothing() {
    let t = sample_repo();
    let readme = write_readme(&t, README);
    gitscope()
        .arg("update")
        .current_dir(t.path())
        .assert()
        .success();
    let first = fs::read(&readme).unwrap();

    gitscope()
        .arg("update")
        .current_dir(t.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("is already up to date"));

    assert_eq!(
        fs::read(&readme).unwrap(),
        first,
        "the file must be byte-identical"
    );
    assert_no_temp_files(t.path());
}

#[test]
fn a_later_update_uses_the_new_history() {
    let t = sample_repo();
    let readme = write_readme(&t, README);
    gitscope()
        .arg("update")
        .current_dir(t.path())
        .assert()
        .success();
    assert!(read(&readme).contains("Total commits: 3\n"));

    t.by("Carol", "carol@example.com")
        .at("2026-04-01 10:00")
        .write("extra.rs", "//\n")
        .commit();

    gitscope()
        .arg("update")
        .current_dir(t.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated"));
    let updated = read(&readme);
    assert!(updated.contains("Total commits: 4\n"));
    assert!(updated.contains("Contributors: 3\n"));
    assert!(updated.contains("Latest commit: 2026-04-01\n"));
    assert_eq!(updated.matches("gitscope:template").count(), 1);
}

#[test]
fn update_accepts_an_explicit_file_and_repository() {
    let t = sample_repo();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("STATS.md");
    fs::write(
        &file,
        "<!-- gitscope:start -->\n{commits} commits\n<!-- gitscope:end -->\n",
    )
    .unwrap();

    gitscope()
        .arg("update")
        .arg(&file)
        .arg("--repo")
        .arg(t.path())
        .assert()
        .success();

    assert!(read(&file).contains("3 commits\n<!-- gitscope:end -->"));
}

#[test]
fn filters_shape_the_values() {
    let t = sample_repo();
    let readme = write_readme(&t, README);

    gitscope()
        .args(["update", "--author", "alice", "--since", "2026-02-01"])
        .current_dir(t.path())
        .assert()
        .success();

    let updated = read(&readme);
    assert!(updated.contains("Total commits: 1\n"), "{updated}");
    assert!(updated.contains("Contributors: 1\n"));
    assert!(updated.contains("Project started: 2026-03-07\n"));
}

#[test]
fn a_missing_readme_is_reported_with_its_path() {
    let t = sample_repo();
    gitscope()
        .arg("update")
        .current_dir(t.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("error: cannot read"))
        .stderr(predicate::str::contains("README.md"));
}

#[test]
fn a_readme_without_a_block_is_left_untouched() {
    let t = sample_repo();
    let original = "# Plain\n\nWe have {commits} commits.\n";
    let readme = write_readme(&t, original);

    gitscope()
        .arg("update")
        .current_dir(t.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("no GitScope block found"))
        .stderr(predicate::str::contains("<!-- gitscope:start -->"));

    assert_eq!(read(&readme), original);
}

#[test]
fn an_unknown_variable_stops_everything_and_leaves_the_file_untouched() {
    let t = sample_repo();
    let original = "<!-- gitscope:start -->\n{commits} {something_that_does_not_exist}\n<!-- gitscope:end -->\n";
    let readme = write_readme(&t, original);

    gitscope()
        .arg("update")
        .current_dir(t.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "error: Unknown GitScope variable: {something_that_does_not_exist} (line 2)",
        ))
        .stderr(predicate::str::contains("Available variables:"))
        .stderr(predicate::str::contains("{first_commit}"));

    assert_eq!(read(&readme), original);
    assert_no_temp_files(t.path());
}

#[test]
fn malformed_blocks_are_rejected_and_the_file_is_untouched() {
    let t = sample_repo();
    for (name, original, expected) in [
        (
            "unclosed",
            "a\n<!-- gitscope:start -->\n{commits}\n",
            "line 2: <!-- gitscope:start --> is never closed",
        ),
        (
            "orphan end",
            "{commits}\n<!-- gitscope:end -->\n",
            "line 2: found <!-- gitscope:end --> without a matching",
        ),
        (
            "duplicated start",
            "<!-- gitscope:start -->\n<!-- gitscope:start -->\n{commits}\n<!-- gitscope:end -->\n",
            "line 2: <!-- gitscope:start --> found inside the block opened at line 1",
        ),
    ] {
        let readme = write_readme(&t, original);
        gitscope()
            .arg("update")
            .current_dir(t.path())
            .assert()
            .failure()
            .code(1)
            .stderr(predicate::str::contains(expected));
        assert_eq!(read(&readme), original, "{name}");
    }
}

#[test]
fn keep_unknown_updates_what_it_can_and_warns() {
    let t = sample_repo();
    let readme = write_readme(
        &t,
        "<!-- gitscope:start -->\n{commits} {nope}\n<!-- gitscope:end -->\n",
    );

    gitscope()
        .args(["update", "--keep-unknown"])
        .current_dir(t.path())
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "warning: left {nope} untouched (line 2)",
        ));

    assert!(read(&readme).contains("\n3 {nope}\n<!-- gitscope:end -->"));
}

#[test]
fn crlf_files_stay_crlf() {
    let t = sample_repo();
    let readme = write_readme(&t, &README.replace('\n', "\r\n"));

    gitscope()
        .arg("update")
        .current_dir(t.path())
        .assert()
        .success();

    let bytes = fs::read(&readme).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    assert!(
        !text.replace("\r\n", "").contains('\n'),
        "a bare LF slipped in"
    );
    assert!(text.contains("Total commits: 3\r\n"));
}

#[test]
fn update_needs_an_explicit_file_for_remote_repositories() {
    let t = sample_repo();
    let dir = tempfile::tempdir().unwrap();
    let url = format!("file:///{}", t.path().to_str().unwrap().replace('\\', "/"));
    let url = url.replacen("file:////", "file:///", 1);

    gitscope()
        .args(["update", "--repo"])
        .arg(&url)
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("no working directory"))
        .stderr(predicate::str::contains("gitscope update FILE"));
}

#[test]
fn markdown_prints_the_resolved_document_and_never_touches_the_template() {
    let t = sample_repo();
    let template = write_readme(&t, README);

    let output = gitscope()
        .arg("markdown")
        .arg(&template)
        .arg("--repo")
        .arg(t.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let printed = String::from_utf8(output.stdout).unwrap();

    let expected = README
        .replace(
            "Project started: {first_commit}",
            "Project started: 2026-01-05",
        )
        .replace("Latest commit: {last_commit}", "Latest commit: 2026-03-07")
        .replace("Total commits: {commits}", "Total commits: 3")
        .replace("Contributors: {contributors}", "Contributors: 2")
        .replace("Files: {files}\n\nLines", "Files: 2\n\nLines")
        .replace("Lines added: {insertions}", "Lines added: 4")
        .replace("Lines deleted: {deletions}", "Lines deleted: 2");
    assert_eq!(printed, expected);
    assert_eq!(read(&template), README);
}

#[test]
fn a_file_without_blocks_is_resolved_as_a_whole() {
    let t = sample_repo();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("template.md");
    fs::write(
        &file,
        "🚀 {commits} commits since {first_commit}\n\n| Metric | Value |\n|---|---:|\n| Commits | {commits} |\n| Files | {files} |\n\n<p>{contributors} people</p>\n",
    )
    .unwrap();

    gitscope()
        .arg("markdown")
        .arg(&file)
        .arg("--repo")
        .arg(t.path())
        .assert()
        .success()
        .stdout(
            "🚀 3 commits since 2026-01-05\n\n| Metric | Value |\n|---|---:|\n| Commits | 3 |\n| Files | 2 |\n\n<p>2 people</p>\n",
        );
}

#[test]
fn markdown_can_write_to_another_file() {
    let t = sample_repo();
    let dir = tempfile::tempdir().unwrap();
    let template = dir.path().join("README.template.md");
    let output = dir.path().join("README.md");
    fs::write(&template, "Commits: {commits}\n").unwrap();

    gitscope()
        .arg("markdown")
        .arg(&template)
        .arg("--repo")
        .arg(t.path())
        .arg("-o")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("wrote"));

    assert_eq!(read(&output), "Commits: 3\n");
    assert_eq!(read(&template), "Commits: {commits}\n");

    gitscope()
        .arg("markdown")
        .arg(&template)
        .arg("--repo")
        .arg(t.path())
        .args(["-o", "-"])
        .assert()
        .success()
        .stdout("Commits: 3\n");
}

#[test]
fn markdown_refuses_to_overwrite_its_own_template() {
    let t = sample_repo();
    let dir = tempfile::tempdir().unwrap();
    let template = dir.path().join("README.md");
    fs::write(&template, "Commits: {commits}\n").unwrap();

    gitscope()
        .arg("markdown")
        .arg(&template)
        .arg("--repo")
        .arg(t.path())
        .arg("--output")
        .arg(&template)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "refusing to overwrite the template",
        ))
        .stderr(predicate::str::contains("gitscope update"));

    assert_eq!(read(&template), "Commits: {commits}\n");
}

#[test]
fn markdown_reports_unknown_variables_without_writing_anything() {
    let t = sample_repo();
    let dir = tempfile::tempdir().unwrap();
    let template = dir.path().join("t.md");
    let output = dir.path().join("out.md");
    fs::write(&template, "{comits}\n").unwrap();

    gitscope()
        .arg("markdown")
        .arg(&template)
        .arg("--repo")
        .arg(t.path())
        .arg("-o")
        .arg(&output)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Unknown GitScope variable: {comits}",
        ))
        .stderr(predicate::str::contains("Did you mean {commits}?"));

    assert!(!output.exists());
}

#[test]
fn the_former_marker_names_still_work() {
    let t = sample_repo();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("old.md");
    fs::write(
        &file,
        "<!-- gitstats:start -->\n{commits}\n<!-- gitstats:end -->\n",
    )
    .unwrap();

    gitscope()
        .arg("markdown")
        .arg(&file)
        .arg("--repo")
        .arg(t.path())
        .assert()
        .success()
        .stdout("<!-- gitstats:start -->\n3\n<!-- gitstats:end -->\n");
}

fn json_report(repo: &TestRepo) -> Value {
    let output = gitscope()
        .arg(repo.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .clone();
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn every_variable_agrees_with_the_json_report() {
    let t = sample_repo();
    let json = json_report(&t);
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("all.md");
    let names = [
        "repository_name",
        "first_commit",
        "last_commit",
        "period",
        "branches",
        "tags",
        "commits",
        "first_commit_message",
        "last_commit_message",
        "most_active_hour",
        "contributors",
        "top_author",
        "top_author_commits",
        "top_author_percentage",
        "files",
        "insertions",
        "deletions",
        "language_count",
        "top_language",
    ];
    let template: String = names.iter().map(|n| format!("{n}={{{n}}}\n")).collect();
    fs::write(&file, template).unwrap();

    let output = gitscope()
        .arg("markdown")
        .arg(&file)
        .arg("--repo")
        .arg(t.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let printed = String::from_utf8(output.stdout).unwrap();
    let value = |name: &str| -> String {
        printed
            .lines()
            .find_map(|l| l.strip_prefix(&format!("{name}=")))
            .unwrap_or_else(|| panic!("{name} missing in {printed}"))
            .to_owned()
    };

    let date = |v: &Value| v.as_str().unwrap()[..10].to_owned();
    let top = &json["contributors"][0];
    assert_eq!(
        value("repository_name"),
        json["repository"]["name"].as_str().unwrap()
    );
    assert_eq!(value("first_commit"), date(&json["commits"]["first"]));
    assert_eq!(value("last_commit"), date(&json["commits"]["last"]));
    assert_eq!(
        value("period"),
        format!(
            "{} → {}",
            date(&json["commits"]["first"]),
            date(&json["commits"]["last"])
        )
    );
    assert_eq!(
        value("branches"),
        json["repository"]["branches"].to_string()
    );
    assert_eq!(value("tags"), json["repository"]["tags"].to_string());
    assert_eq!(value("commits"), json["commits"]["total"].to_string());
    assert_eq!(
        value("first_commit_message"),
        json["commits"]["first_message"].as_str().unwrap()
    );
    assert_eq!(
        value("last_commit_message"),
        json["commits"]["last_message"].as_str().unwrap()
    );
    assert_eq!(
        value("most_active_hour"),
        format!(
            "{:02}:00",
            json["commits"]["most_active_hour"].as_u64().unwrap()
        )
    );
    assert_eq!(
        value("contributors"),
        json["contributors"].as_array().unwrap().len().to_string()
    );
    assert_eq!(value("top_author"), top["name"].as_str().unwrap());
    assert_eq!(value("top_author_commits"), top["commits"].to_string());
    assert_eq!(
        value("top_author_percentage"),
        format!("{:.1}", top["percentage"].as_f64().unwrap())
    );
    assert_eq!(value("files"), json["files"]["total"].to_string());
    assert_eq!(
        value("insertions"),
        json["commits"]["insertions"].to_string()
    );
    assert_eq!(value("deletions"), json["commits"]["deletions"].to_string());
    assert_eq!(
        value("language_count"),
        json["languages"].as_array().unwrap().len().to_string()
    );
    assert_eq!(
        value("top_language"),
        json["languages"][0]["name"].as_str().unwrap()
    );

    let sum = |field: &str| -> u64 {
        json["contributors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c[field].as_u64().unwrap())
            .sum()
    };
    assert_eq!(
        json["commits"]["insertions"].as_u64().unwrap(),
        sum("insertions")
    );
    assert_eq!(
        json["commits"]["deletions"].as_u64().unwrap(),
        sum("deletions")
    );
}

#[test]
fn compound_author_variables_use_real_numbers() {
    let t = sample_repo();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("author.md");
    fs::write(
        &file,
        "Alice made {author:Alice:commits} commits (+{author:Alice:insertions} / -{author:Alice:deletions}); Bob made {author:bob@example.com:commits}.\n",
    )
    .unwrap();

    gitscope()
        .arg("markdown")
        .arg(&file)
        .arg("--repo")
        .arg(t.path())
        .assert()
        .success()
        .stdout("Alice made 2 commits (+2 / -1); Bob made 1.\n");

    fs::write(&file, "{author:Nobody:commits}\n").unwrap();
    gitscope()
        .arg("markdown")
        .arg(&file)
        .arg("--repo")
        .arg(t.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Invalid GitScope variable: {author:Nobody:commits}",
        ))
        .stderr(predicate::str::contains("no contributor named 'Nobody'"));
}

#[test]
fn an_empty_repository_gives_not_available_values() {
    let t = TestRepo::named("fresh");
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("t.md");
    fs::write(
        &file,
        "{commits}|{first_commit}|{top_author}|{top_language}|{repository_name}\n",
    )
    .unwrap();

    gitscope()
        .arg("markdown")
        .arg(&file)
        .arg("--repo")
        .arg(t.path())
        .assert()
        .success()
        .stdout("0|n/a|n/a|n/a|fresh\n");
}

#[test]
fn a_commit_message_cannot_break_the_readme() {
    let t = TestRepo::named("tricky");
    t.by("Alice", "alice@example.com")
        .at("2026-01-05 14:10")
        .message("<!-- gitscope:end --> {commits} --> oops")
        .write("a.rs", "x\n")
        .commit();
    let readme = write_readme(
        &t,
        "<!-- gitscope:start -->\nLast: {last_commit_message}\n<!-- gitscope:end -->\n",
    );

    gitscope()
        .arg("update")
        .current_dir(t.path())
        .assert()
        .success();
    let once = read(&readme);
    assert!(once.contains("Last: &lt;!-- gitscope:end --&gt; {commits} --&gt; oops\n"));

    gitscope()
        .arg("update")
        .current_dir(t.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("already up to date"));
    assert_eq!(read(&readme), once);
}

#[test]
fn variables_lists_everything_without_needing_a_repository() {
    let dir = tempfile::tempdir().unwrap();
    gitscope()
        .arg("variables")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Repository\n"))
        .stdout(predicate::str::contains("Contributors\n"))
        .stdout(predicate::str::is_match(r"\{commits\}\s+Number of commits").unwrap())
        .stdout(predicate::str::contains(
            "{author:NAME:commits|insertions|deletions}",
        ))
        .stdout(predicate::str::contains("escape it as \\{name}"));
}

#[test]
fn the_default_report_still_works_and_a_path_named_like_a_command_is_reachable() {
    let t = sample_repo();
    gitscope()
        .arg(t.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("GITSCOPE v"))
        .stdout(predicate::str::contains("Top contributors"));

    gitscope()
        .args(["update", "--json"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn help_mentions_the_markdown_commands() {
    gitscope()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("markdown"))
        .stdout(predicate::str::contains("variables"));
    gitscope()
        .args(["update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--keep-unknown"))
        .stdout(predicate::str::contains("README.md"));
}

#[test]
fn every_variable_is_documented_in_both_languages() {
    let docs = [
        ("docs/MARKDOWN.md", include_str!("../docs/MARKDOWN.md")),
        (
            "docs/MARKDOWN-EN.md",
            include_str!("../docs/MARKDOWN-EN.md"),
        ),
    ];
    let registry = gitscope::output::variables::VariableRegistry::standard();
    for (file, text) in docs {
        for def in registry.iter() {
            let plain = format!("{{{}}}", def.name);
            let compound = format!("{{{}:", def.name);
            assert!(
                text.contains(&plain) || text.contains(&compound),
                "{file} does not mention {}",
                def.usage()
            );
        }
    }
}
