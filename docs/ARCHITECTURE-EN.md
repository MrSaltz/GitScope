# Architecture

[Português](ARCHITECTURE.md) · **English**

GitScope is a small pipeline. Each stage has one job and knows as little as possible about the others.

```
Git repository
      │
      ▼
repository.rs      Data collection (open, clone, walk, diff). The only module that
      │            imports git2. Yields CommitRecord values one at a time.
      │            (remote.rs is pure URL parsing that decides local vs. clone.)
      ▼
model.rs           Domain types (CommitRecord, RepositoryStats, ...). No git2.
      │
      ▼
analysis/          Pure statistics over CommitRecords. Never prints, never touches Git.
      │
      ├──────────────┐
      ▼              ▼
output/terminal.rs   output/json.rs   output/markdown.rs (+ variables.rs)   output/tui/
                                      Formatting only. Never recomputes anything.
```

`cli.rs` defines the arguments (clap), `error.rs` the domain errors (thiserror), and `main.rs` glues everything together and turns errors into exit codes (anyhow).

## Rules that keep it maintainable

1. **`git2` stays in `repository.rs`.** Everything else works on `model` types. This is what lets `analysis/` be tested with hand-written data and lets a future backend (or a mock) replace Git.
2. **`analysis/` is pure.** No I/O, no printing.
3. **`output/` only formats.** If a renderer needs a number, the analysis must produce it; the JSON schema is literally the serialised [`RepositoryStats`](../src/model.rs).
4. **No `git` executable.** Never shell out to `git`. Use `git2` (libgit2).
5. **Nothing left behind, nothing leaked.** A temporary clone is owned by the `Repository` handle and deleted when it is closed or dropped. Credentials inside URLs are redacted (`remote::redact`) before any message is built.
6. **Incremental.** Commits flow through the pipeline one at a time. A `Diff` never outlives one loop iteration, and nothing is retained per commit unless `--all` asks for it.

## Data flow in detail

1. `open_source` (in `lib.rs`) opens a local path or, for a URL that does not exist on disk, clones it (bare, into a temporary directory) with `Repository::clone_to_temp`. `analyze_repository` validates the [`Filters`](../src/model.rs) and reads the repository-level facts (name, branches, tags). `analyze_path` is the local-only shorthand.
2. `Repository::commits` walks history from the tip (`HEAD` or `--branch`) with a `revwalk`. For each commit it reads the author and date and applies the **author and date filters first**, before doing any diff work. Surviving commits are diffed against their first parent (rename detection on) and turned into a `CommitRecord`. Merge commits are not diffed.
3. `analysis::analyze` feeds each `CommitRecord` to a set of *accumulators*, one per topic:

   | Module | Accumulates |
   |---|---|
   | `commits.rs` | totals, first/last, weekday and hour buckets |
   | `contributors.rs` | per-identity commits, files, lines, dates |
   | `files.rs` | modification count per path; extension counts of the snapshot |
   | `activity.rs` | commits per month |
   | `languages.rs` | language shares of the snapshot (table-driven) |

   With `Options { commit_details: true }` (`--all`) it also keeps a `CommitInfo` per commit.
4. `Repository::snapshot_files` lists the file tree at the tip; it feeds the snapshot-based statistics.
5. The result, `RepositoryStats`, is handed to `output::terminal::render` or `output::json::render`.
6. For Markdown, `output::markdown` splits the document into lines, finds and validates the `gitscope` blocks, resolves every `{variable}` through the `VariableRegistry` (`output/variables.rs`, which only *reads* `RepositoryStats`) and rebuilds the document, changing nothing but the variables. `update_file` then writes it atomically.
7. For the interface, `main.rs` analyses once (with `commit_details`) and hands the result to `output::tui::run`. `tui/app.rs` is the state (tab, selections, commit filter) and the key handling, a plain value with no drawing; `tui/ui.rs` draws it with `ratatui`; `tui/mod.rs` owns the terminal and the event loop. `tui/i18n.rs` holds the interface texts, one complete `Texts` struct per language, and `config.rs` reads and writes the small JSON file where the choices (the language) are kept.

## Extending GitScope

### Add a language

Add one line to `LANGUAGES` in [`src/analysis/languages.rs`](../src/analysis/languages.rs):

```rust
("zig", "Zig"),
```

Extensions are lowercase and unique (a unit test enforces this). The analysis code does not change.

### Add a statistic

1. Add the result type to `model.rs` (derive `Serialize`) and a field on `RepositoryStats`.
2. Write an accumulator in `analysis/` with `add(&CommitRecord)` and `finish()`, plus unit tests using `analysis::testutil`.
3. Call it from `analysis::analyze`.
4. Render it in `output/terminal.rs`. JSON picks it up automatically.
5. This changes the public JSON schema: update [`JSON-EN.md`](JSON-EN.md), the key-set assertions in `tests/cli.rs` and the changelog.

### Add a Markdown variable

Add one entry to `standard_variables()` in [`src/output/variables.rs`](../src/output/variables.rs):

```rust
plain("Commits", "commits", "Number of commits", |c, _| {
    Ok(thousands(c.stats.commits.total))
}),
```

A name, a group, a description and a resolver. The resolver gets the arguments written after the name (`{author:Alice:commits}` -> `["Alice", "commits"]`), so compound variables need nothing extra; declare them with `VariableDef { arguments: "NAME:FIELD", .. }`. `gitscope variables`, the error messages, the typo suggestions and the tests that check the registry pick it up automatically. Update [`MARKDOWN-EN.md`](MARKDOWN-EN.md): a test fails if a variable is missing from it. If the value needs data the model does not have, add it to the analysis (and to the JSON docs), never to the resolver.

### Add an interface language

Add a variant to `Language` in `src/config.rs` (its `serde` name, its native name and its place in `ALL`), then a complete `Texts` static for it in `src/output/tui/i18n.rs` and a line in `texts()`. The compiler points at everything that is missing, and the tests check the placeholders and that no text is empty.

### Need more data from Git

Extend `CommitRecord` and `repository.rs` only. Keep `git2` types out of the model.

## Testing approach

| Layer | How it is tested |
|---|---|
| `analysis/*`, `output/terminal.rs` helpers, `cli.rs` | Unit tests next to the code, with hand-built values. |
| `repository.rs` + `analysis` | `tests/repository.rs`, `tests/analysis.rs`: real repositories built with `git2` by `tests/common::TestRepo` (fixed authors and timestamps, no `git` executable). |
| End to end | `tests/cli.rs` runs the compiled binary: exit codes, messages, terminal output, JSON schema. |
| Markdown engine | `output/markdown/tests.rs` and `output/variables.rs`: hand-built statistics; blocks, escapes, CRLF/BOM, idempotence, atomic writes, error cases. |
| Markdown commands | `tests/markdown.rs` runs `update`, `markdown` and `variables` on real repositories and cross-checks every variable against the JSON report. |
| Interface | `output/tui/tests.rs`: every key against `App` (no terminal) and every screen drawn on ratatui's in-memory `TestBackend`, including tiny sizes and empty repositories. `tests/cli.rs` checks that it refuses to run without a terminal. |
| Benchmarks | `benches/analysis.rs` (`cargo bench`): builds a synthetic repository with `TestRepo` and times the walk, the full analysis, the analysis of one author and each renderer. It has no framework; it prints the best and median times. |
| Remote | `tests/remote.rs` clones `file://` URLs of `TestRepo`s (no network), points the binary's temp directory at a scratch folder to prove clean-up, and fails against a closed local port. |

## Design decisions worth knowing

- **Author date, UTC.** Deterministic across machines and time zones, and the date a contributor wrote the change.
- **Email is the identity.** Names vary; emails rarely do. `.mailmap` support is a possible extension.
- **Snapshot vs. history.** "How many files / which languages" describes the project as it is; "who did what and when" describes the filtered history. Mixing them (for example filtering the file count by author) would not be meaningful.
- **Bare clone.** For remote analysis only history matters, so the clone has no working tree: faster, less disk, and no hooks or checkout filters can run.
- **Cargo feature `remote`.** HTTPS support pulls in TLS (OpenSSL on Linux). It is on by default and can be disabled (`--no-default-features`); everything local keeps working.
- **`update` keeps the template.** Replacing `{commits}` by `361` destroys the template, and a README that cannot be refreshed is of little use. So an updated block stores its template in an HTML comment (invisible when rendered), like code generators that keep their source next to the generated output. `markdown` is the pure counterpart: it never stores anything and never writes the input.
- **Only variables change.** The engine works on the original text and splices results in; it never reformats, so CRLF, a BOM, tables and HTML survive. Values are neutralised (`<!--`, `-->`) and never rescanned, so data cannot alter the document's structure.
- **Analyse first, browse after.** The interface analyses once before opening (no threads, no loading screen) and then only browses; filtering the commit list hides items that are already loaded. The state has no drawing code, so keys are tested without a terminal.
- **Language is an interface setting.** The texts are a struct per language (a missing translation does not compile) and the choice lives in a small JSON file in the user's configuration directory, written atomically like every other file GitScope writes. The report, the errors and the other commands stay in English.
- **`HEAD` by default.** Coherent with the file snapshot and with `git log`; use `--branch` for another line of history.
- **Performance is measured, not assumed.** See the *Performance* section of the README for measurements and the identified bottleneck.
