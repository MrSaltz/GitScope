use super::*;
use crate::output::variables::tests::{empty_fixture, fixture};

fn run(source: &str, options: Options) -> std::result::Result<Rendered, MarkdownError> {
    render(
        source,
        &VariableRegistry::standard(),
        &Context::new(&fixture()),
        &options,
    )
}

fn render_ok(source: &str) -> String {
    run(source, Options::default()).unwrap().text
}

fn update_options() -> Options {
    Options {
        mode: Mode::Update,
        unknown: UnknownPolicy::Error,
    }
}

fn update_ok(source: &str) -> String {
    run(source, update_options()).unwrap().text
}

fn update_with(source: &str, stats: &crate::model::RepositoryStats) -> String {
    render(
        source,
        &VariableRegistry::standard(),
        &Context::new(stats),
        &update_options(),
    )
    .unwrap()
    .text
}

fn error_of(source: &str, options: Options) -> MarkdownError {
    run(source, options).expect_err("expected an error")
}

#[test]
fn a_valid_variable_is_replaced() {
    assert_eq!(render_ok("Commits: {commits}\n"), "Commits: 3\n");
}

#[test]
fn repeated_and_multiple_variables() {
    assert_eq!(
        render_ok("{commits} / {commits} / {files} files by {contributors}"),
        "3 / 3 / 3 files by 2"
    );
}

#[test]
fn only_the_variables_change_whatever_the_formatting() {
    let source = "\
Início: {first_commit}

Início do projeto → {first_commit}

> Project started on {first_commit}

**Projeto iniciado em {first_commit}**

🚀 This project has {commits} commits from {contributors} contributors.
";
    let expected = "\
Início: 2026-01-05

Início do projeto → 2026-01-05

> Project started on 2026-01-05

**Projeto iniciado em 2026-01-05**

🚀 This project has 3 commits from 2 contributors.
";
    assert_eq!(render_ok(source), expected);
}

#[test]
fn tables_and_html_are_preserved() {
    let source = "\
| Metric | Value |
|---|---:|
| Commits | {commits} |
| Contributors | {contributors} |
| Files | {files} |

<p align=\"center\"><b>{commits}</b> commits &middot; <a href=\"https://example.com/?a=1&b=2\">link</a></p>
<img src=\"badge.svg\" alt=\"{top_language}\">
";
    let expected = "\
| Metric | Value |
|---|---:|
| Commits | 3 |
| Contributors | 2 |
| Files | 3 |

<p align=\"center\"><b>3</b> commits &middot; <a href=\"https://example.com/?a=1&b=2\">link</a></p>
<img src=\"badge.svg\" alt=\"Rust\">
";
    assert_eq!(render_ok(source), expected);
}

#[test]
fn braces_that_are_not_variables_are_left_alone() {
    let source = "\
{}
{ commits }
{not a variable}
{\"json\": true, \"n\": 1}
a { color: red; }
${HOME} and ${commits}
{{commits}}
\\{ x \\}
{9lives}
{
";
    assert_eq!(render_ok(source), source);
}

#[test]
fn a_variable_can_be_escaped() {
    assert_eq!(
        render_ok("Write \\{commits} to get {commits}."),
        "Write {commits} to get 3."
    );
    assert_eq!(render_ok("C:\\{tmp \\d"), "C:\\{tmp \\d");
}

#[test]
fn values_are_inserted_once_and_never_rescanned() {
    let mut stats = fixture();
    stats.commits.last_message = Some("mentions {commits} and {nope}".into());
    let out = update_with(
        "<!-- gitscope:start -->\n{last_commit_message}\n<!-- gitscope:end -->\n",
        &stats,
    );
    assert!(out.contains("\nmentions {commits} and {nope}\n<!-- gitscope:end -->"));
}

#[test]
fn html_comments_inside_values_cannot_break_the_document() {
    let mut stats = fixture();
    stats.commits.last_message = Some("<!-- gitscope:end --> oops --> <!-- x".into());
    let source = "<!-- gitscope:start -->\n{last_commit_message}\n<!-- gitscope:end -->\n";

    let out = update_with(source, &stats);
    assert!(out.contains("&lt;!-- gitscope:end --&gt; oops --&gt; &lt;!-- x"));
    assert_eq!(update_with(&out, &stats), out);
}

#[test]
fn empty_values_read_not_available() {
    let out = render(
        "{first_commit}|{top_author}|{top_language}|{commits}|{most_active_hour}",
        &VariableRegistry::standard(),
        &Context::new(&empty_fixture()),
        &Options::default(),
    )
    .unwrap()
    .text;
    assert_eq!(out, "n/a|n/a|n/a|0|n/a");
}

#[test]
fn compound_variables_work_in_text() {
    assert_eq!(
        render_ok("Alice made {author:Alice:commits} commits (+{author:alice:insertions})."),
        "Alice made 2 commits (+1,010)."
    );
}

#[test]
fn an_unknown_variable_is_an_error_that_lists_the_alternatives() {
    let error = error_of("Hello {something_that_does_not_exist}", Options::default());
    let text = error.to_string();
    assert!(text.contains("Unknown GitScope variable: {something_that_does_not_exist} (line 1)"));
    assert!(text.contains("Available variables:"));
    assert!(text.contains("{commits}"));
    assert!(text.contains("{author:NAME:commits|insertions|deletions}"));
}

#[test]
fn a_typo_gets_a_suggestion() {
    let text = error_of("{comits}", Options::default()).to_string();
    assert!(text.contains("Did you mean {commits}?"), "{text}");
    let text = error_of("{Commits}", Options::default()).to_string();
    assert!(text.contains("Did you mean {commits}?"), "{text}");
}

#[test]
fn every_problem_is_reported_with_its_line() {
    let source = "ok {commits}\n\n{nope1}\n<!-- gitscope:start -->\n{nope2} {author:Nobody:commits}\n<!-- gitscope:end -->\n";
    let MarkdownError::Variables { problems, .. } = error_of(source, Options::default()) else {
        panic!("expected variable problems");
    };
    let found: Vec<_> = problems
        .iter()
        .map(|p| (p.line, p.token.as_str()))
        .collect();
    assert_eq!(found, vec![(5, "{nope2}"), (5, "{author:Nobody:commits}")]);
    assert!(matches!(&problems[1].kind, ProblemKind::Invalid(m) if m.contains("no contributor")));
}

#[test]
fn misused_variables_are_errors_too() {
    let text = error_of("{commits:1}", Options::default()).to_string();
    assert!(
        text.contains(
            "Invalid GitScope variable: {commits:1} (line 1): {commits} takes no arguments"
        )
    );
}

#[test]
fn keep_unknown_leaves_them_untouched_and_warns() {
    let options = Options {
        mode: Mode::Render,
        unknown: UnknownPolicy::Keep,
    };
    let rendered = run("{commits} {nope} {author:Nobody:commits}", options).unwrap();
    assert_eq!(rendered.text, "3 {nope} {author:Nobody:commits}");
    assert_eq!(rendered.replacements, 1);
    assert_eq!(rendered.warnings.len(), 2);
    assert_eq!(rendered.warnings[0].token, "{nope}");
}

const BLOCK: &str = "\
# Title

Outside: {commits} stays as written.

<!-- gitscope:start -->

Inside: {commits}

<!-- gitscope:end -->

After: {files} stays too.
";

#[test]
fn only_the_inside_of_a_block_is_modified() {
    let expected = "\
# Title

Outside: {commits} stays as written.

<!-- gitscope:start -->

Inside: 3

<!-- gitscope:end -->

After: {files} stays too.
";
    assert_eq!(render_ok(BLOCK), expected);
}

#[test]
fn several_blocks_are_handled_independently() {
    let source = "a {x}\n<!-- gitscope:start -->\n{commits}\n<!-- gitscope:end -->\nb {x}\n<!-- gitscope:start -->\n{files}\n<!-- gitscope:end -->\nc {x}\n";
    let rendered = run(source, Options::default()).unwrap();
    assert_eq!(
        rendered.text,
        "a {x}\n<!-- gitscope:start -->\n3\n<!-- gitscope:end -->\nb {x}\n<!-- gitscope:start -->\n3\n<!-- gitscope:end -->\nc {x}\n"
    );
    assert_eq!((rendered.blocks, rendered.replacements), (2, 2));
}

#[test]
fn marker_spelling_is_flexible_and_the_previous_name_still_works() {
    for (start, end) in [
        ("<!-- gitscope:start -->", "<!-- gitscope:end -->"),
        ("<!--gitscope:start-->", "<!--gitscope:end-->"),
        ("<!--   gitscope:start   -->  ", "   <!-- gitscope:end -->"),
        ("<!-- gitstats:start -->", "<!-- gitstats:end -->"),
        ("<!-- gitstats:start -->", "<!-- gitscope:end -->"),
        ("<!-- gitstatus:start -->", "<!-- gitstatus:end -->"),
    ] {
        let source = format!("{start}\n{{commits}}\n{end}\n");
        assert_eq!(
            render_ok(&source),
            format!("{start}\n3\n{end}\n"),
            "{start}"
        );
    }
}

#[test]
fn an_unclosed_block_is_rejected() {
    let error = error_of(
        "a\n<!-- gitscope:start -->\n{commits}\n",
        Options::default(),
    );
    assert_eq!(error, MarkdownError::UnclosedBlock { line: 2 });
    assert!(error.to_string().contains("line 2"));
}

#[test]
fn an_end_without_start_is_rejected() {
    let error = error_of("{commits}\n<!-- gitscope:end -->\n", Options::default());
    assert_eq!(error, MarkdownError::UnmatchedEnd { line: 2 });
}

#[test]
fn duplicated_or_nested_starts_are_rejected() {
    let source = "<!-- gitscope:start -->\n{commits}\n<!-- gitscope:start -->\n{files}\n<!-- gitscope:end -->\n";
    assert_eq!(
        error_of(source, Options::default()),
        MarkdownError::NestedBlock {
            line: 3,
            open_line: 1
        }
    );
    let doubled_end =
        "<!-- gitscope:start -->\n{commits}\n<!-- gitscope:end -->\n<!-- gitscope:end -->\n";
    assert_eq!(
        error_of(doubled_end, Options::default()),
        MarkdownError::UnmatchedEnd { line: 4 }
    );
}

#[test]
fn without_blocks_render_treats_the_whole_document_as_the_template() {
    let rendered = run("# T\n\n{commits} commits\n", Options::default()).unwrap();
    assert_eq!(rendered.text, "# T\n\n3 commits\n");
    assert_eq!(rendered.blocks, 0);
}

#[test]
fn without_blocks_update_refuses_to_run() {
    assert_eq!(
        error_of("# T\n{commits}\n", update_options()),
        MarkdownError::NoBlocks
    );
}

#[test]
fn markers_in_code_are_documentation_not_markers() {
    let source = "\
Use the markers like this:

```markdown
<!-- gitscope:start -->
{commits}
```

and `<!-- gitscope:start -->` inline, or indented:

    <!-- gitscope:end -->

~~~
<!-- gitscope:end -->
~~~

<!-- gitscope:start -->
{commits}
<!-- gitscope:end -->
";
    let out = update_ok(source);
    assert!(
        out.starts_with(
            &source[..source
                .find("<!-- gitscope:start -->\n{commits}\n<!-- gitscope:end -->")
                .unwrap()]
        )
    );
    assert!(out.contains("```markdown\n<!-- gitscope:start -->\n{commits}\n```"));
    assert!(out.ends_with("<!-- gitscope:end -->\n"));
    assert_eq!(run(source, update_options()).unwrap().blocks, 1);
}

const README: &str = "\
# Lynox OS

Operating system built from scratch.

## Statistics

<!-- gitscope:start -->

Project started: {first_commit}

Total commits: {commits}

<!-- gitscope:end -->

Footer with {commits} untouched.
";

#[test]
fn update_keeps_the_template_and_shows_the_values() {
    let expected = "\
# Lynox OS

Operating system built from scratch.

## Statistics

<!-- gitscope:start -->
<!-- gitscope:template

Project started: {first_commit}

Total commits: {commits}

-->

Project started: 2026-01-05

Total commits: 3

<!-- gitscope:end -->

Footer with {commits} untouched.
";
    assert_eq!(update_ok(README), expected);
}

#[test]
fn a_block_stored_by_an_older_version_is_migrated_to_the_current_name() {
    let old = "<!-- gitstatus:start -->
<!-- gitstatus:template
Commits: {commits}
-->
Commits: 999
<!-- gitstatus:end -->
";
    let updated = update_ok(old);
    assert_eq!(
        updated,
        "<!-- gitstatus:start -->
<!-- gitscope:template
Commits: {commits}
-->
Commits: 3
<!-- gitstatus:end -->
"
    );
    assert_eq!(update_ok(&updated), updated);
}

#[test]
fn updating_twice_gives_exactly_the_same_document() {
    let once = update_ok(README);
    let twice = update_ok(&once);
    assert_eq!(once, twice);
    assert_eq!(update_ok(&twice), once);
}

#[test]
fn an_update_reads_current_data_from_the_stored_template() {
    let with_data = update_ok(README);
    let empty = update_with(&with_data, &empty_fixture());
    assert!(empty.contains("Project started: n/a"));
    assert!(empty.contains("Total commits: 0"));
    assert!(empty.contains("Project started: {first_commit}"));
    assert_eq!(update_ok(&empty), with_data);
}

#[test]
fn editing_the_stored_template_is_honoured() {
    let mut doc = update_ok(README);
    doc = doc.replace(
        "Total commits: {commits}\n\n-->",
        "Total commits: {commits} by {contributors}\n\n-->",
    );
    let updated = update_ok(&doc);
    assert!(updated.contains("Total commits: 3 by 2"));
}

#[test]
fn render_mode_turns_an_updated_document_into_a_clean_one() {
    let stored = update_ok(README);
    let clean = render_ok(&stored);
    assert!(!clean.contains("gitscope:template"));
    assert!(clean.contains("Project started: 2026-01-05"));
    assert!(clean.contains("Footer with {commits} untouched."));
}

#[test]
fn blocks_with_nothing_to_substitute_are_left_as_written() {
    let source = "<!-- gitscope:start -->\n\nNo variables here.\n\n<!-- gitscope:end -->\n<!-- gitscope:start -->\n<!-- gitscope:end -->\n<!-- gitscope:start -->\n\n<!-- gitscope:end -->\n";
    assert_eq!(update_ok(source), source);
}

#[test]
fn an_escape_is_stored_so_it_stays_literal_on_the_next_update() {
    let source = "<!-- gitscope:start -->\nWrite \\{commits}: {commits}\n<!-- gitscope:end -->\n";
    let once = update_ok(source);
    assert!(once.contains("Write {commits}: 3\n<!-- gitscope:end -->"));
    assert!(once.contains("Write \\{commits}: {commits}\n-->"));
    assert_eq!(update_ok(&once), once);
}

#[test]
fn a_template_with_an_html_comment_cannot_be_stored_but_can_be_rendered() {
    let source = "<!-- gitscope:start -->\n{commits} <!-- note -->\n<!-- gitscope:end -->\n";
    assert_eq!(
        error_of(source, update_options()),
        MarkdownError::UnstorableTemplate { line: 1 }
    );
    assert_eq!(
        render_ok(source),
        "<!-- gitscope:start -->\n3 <!-- note -->\n<!-- gitscope:end -->\n"
    );
}

#[test]
fn an_unterminated_stored_template_is_rejected() {
    let source =
        "<!-- gitscope:start -->\n<!-- gitscope:template\n{commits}\n<!-- gitscope:end -->\n";
    assert_eq!(
        error_of(source, update_options()),
        MarkdownError::UnterminatedTemplate { line: 2 }
    );
}

#[test]
fn an_error_in_any_block_means_no_output_at_all() {
    let source = "<!-- gitscope:start -->\n{commits}\n<!-- gitscope:end -->\n<!-- gitscope:start -->\n{nope}\n<!-- gitscope:end -->\n";
    assert!(run(source, update_options()).is_err());
}

#[test]
fn crlf_documents_stay_crlf() {
    let source = README.replace('\n', "\r\n");
    let out = update_ok(&source);
    assert!(
        !out.replace("\r\n", "").contains('\n'),
        "found a bare LF in {out:?}"
    );
    assert!(out.contains("<!-- gitscope:template\r\n"));
    assert_eq!(update_ok(&out), out);
    assert_eq!(
        render_ok(&source),
        README
            .replace('\n', "\r\n")
            .replace("{first_commit}", "2026-01-05")
            .replace("Total commits: {commits}", "Total commits: 3")
            .replacen("Footer with 3", "Footer with {commits}", 1)
    );
}

#[test]
fn a_missing_final_newline_and_a_bom_are_preserved() {
    let source = "\u{feff}<!-- gitscope:start -->\n{commits}\n<!-- gitscope:end -->";
    let out = update_ok(source);
    assert!(out.starts_with('\u{feff}'));
    assert!(out.ends_with("<!-- gitscope:end -->"));
    assert_eq!(update_ok(&out), out);
}

#[test]
fn an_empty_document_is_fine() {
    assert_eq!(render_ok(""), "");
    assert_eq!(error_of("", update_options()), MarkdownError::NoBlocks);
}

#[test]
fn the_output_is_deterministic() {
    let outputs: Vec<String> = (0..5).map(|_| update_ok(README)).collect();
    assert!(outputs.windows(2).all(|w| w[0] == w[1]));
}

#[test]
fn the_documented_example_works_exactly() {
    let before = "\
# Lynox OS

Operating system built from scratch.

## Statistics

<!-- gitstats:start -->

Project started: {first_commit}

Latest commit: {last_commit}

Total commits: {commits}

Contributors: {contributors}

Files: {files}

Lines added: {insertions}

Lines deleted: {deletions}

<!-- gitstats:end -->
";
    let after = "\
# Lynox OS

Operating system built from scratch.

## Statistics

<!-- gitstats:start -->

Project started: 2026-01-05

Latest commit: 2026-03-07

Total commits: 3

Contributors: 2

Files: 3

Lines added: 1,012

Lines deleted: 6

<!-- gitstats:end -->
";
    assert_eq!(render_ok(before), after);
}

fn context_registry() -> (VariableRegistry, crate::model::RepositoryStats) {
    (VariableRegistry::standard(), fixture())
}

fn leftovers(dir: &Path) -> Vec<String> {
    fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn write_atomic_replaces_the_file_and_leaves_no_temporary_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("README.md");
    fs::write(&path, "old").unwrap();

    write_atomic(&path, "new content\n").unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "new content\n");
    assert_eq!(leftovers(dir.path()), vec!["README.md".to_owned()]);
}

#[cfg(unix)]
#[test]
fn write_atomic_keeps_permissions_and_follows_symlinks() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("real.md");
    fs::write(&real, "old").unwrap();
    fs::set_permissions(&real, fs::Permissions::from_mode(0o644)).unwrap();
    let link = dir.path().join("README.md");
    std::os::unix::fs::symlink(&real, &link).unwrap();

    write_atomic(&link, "new").unwrap();

    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read_to_string(&real).unwrap(), "new");
    assert_eq!(
        fs::metadata(&real).unwrap().permissions().mode() & 0o777,
        0o644
    );
}

#[test]
fn update_file_rewrites_only_when_something_changes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("README.md");
    fs::write(&path, README).unwrap();
    let (registry, stats) = context_registry();
    let context = Context::new(&stats);

    let first = update_file(&path, &registry, &context, UnknownPolicy::Error).unwrap();
    assert!(first.changed);
    assert_eq!((first.blocks, first.replacements), (1, 2));
    let after_first = fs::read_to_string(&path).unwrap();
    assert_eq!(after_first, update_ok(README));

    let second = update_file(&path, &registry, &context, UnknownPolicy::Error).unwrap();
    assert!(!second.changed);
    assert_eq!(fs::read_to_string(&path).unwrap(), after_first);
    assert_eq!(leftovers(dir.path()), vec!["README.md".to_owned()]);
}

#[test]
fn a_failed_update_leaves_the_original_file_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("README.md");
    let broken = "<!-- gitscope:start -->\n{commits}\n<!-- gitscope:end -->\n<!-- gitscope:start -->\n{nope}\n<!-- gitscope:end -->\n";
    fs::write(&path, broken).unwrap();
    let (registry, stats) = context_registry();

    let error = update_file(
        &path,
        &registry,
        &Context::new(&stats),
        UnknownPolicy::Error,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        GitScopeError::Markdown(MarkdownError::Variables { .. })
    ));
    assert_eq!(fs::read_to_string(&path).unwrap(), broken);
    assert_eq!(leftovers(dir.path()), vec!["README.md".to_owned()]);
}

#[test]
fn a_malformed_file_is_left_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("README.md");
    let broken = "<!-- gitscope:start -->\n{commits}\n";
    fs::write(&path, broken).unwrap();
    let (registry, stats) = context_registry();

    let error = update_file(
        &path,
        &registry,
        &Context::new(&stats),
        UnknownPolicy::Error,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        GitScopeError::Markdown(MarkdownError::UnclosedBlock { line: 1 })
    ));
    assert_eq!(fs::read_to_string(&path).unwrap(), broken);
}

#[test]
fn reading_problems_name_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("nope.md");
    let error = read_document(&missing).unwrap_err().to_string();
    assert!(error.starts_with("cannot read "), "{error}");
    assert!(error.contains("nope.md"), "{error}");

    let binary = dir.path().join("binary.md");
    fs::write(&binary, [0xff, 0xfe, 0x00, 0x80]).unwrap();
    assert!(read_document(&binary).is_err());
}
