# Changelog

[Português](CHANGELOG.md) · **English**

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses [Semantic Versioning](https://semver.org/) (before 1.0, minor versions may change the CLI).

The JSON output has its own version, `schema_version`; see [docs/JSON-EN.md](docs/JSON-EN.md).

## [1.0.0] - 2026-09-20

The first stable release: the command line, the exit codes, the JSON schema, the Markdown template syntax and the configuration file now follow Semantic Versioning (see [docs/STABILITY-EN.md](docs/STABILITY-EN.md)).

### Added

- **Stability policy** ([docs/STABILITY-EN.md](docs/STABILITY-EN.md)): what the version number promises, what it does not, and how breaking changes are handled.
- **Benchmarks** (`cargo bench`, `benches/analysis.rs`): a synthetic repository, timing of the walk, the full analysis, one author and each renderer. No framework and no new dependency.
- **Release workflow** (`.github/workflows/release.yml`): on a `vX.Y.Z` tag it checks the version, builds binaries for Linux x86_64, Windows x86_64 and macOS (arm64 and x86_64), and creates the GitHub Release with the archives and `SHA256SUMS` (tags with a hyphen become pre-releases). It has not been run yet; the first tag is its test (see [docs/RELEASING-EN.md](docs/RELEASING-EN.md)).
- Cargo feature `vendored-openssl`, which bundles OpenSSL (used by the Linux release binary).
- Installation documentation for the pre-built binaries, and the release process ([docs/RELEASING-EN.md](docs/RELEASING-EN.md)).

### Changed

- `GitScopeError`, `MarkdownError`, `ProblemKind` and `ChangeKind` are `non_exhaustive`, so adding a variant later is not a breaking change.
- The crate is packaged without the `.github/` folder; `cargo publish --dry-run` succeeds (55 files). `repository` and `homepage` are still to be filled in.

## [0.8.0] - 2026-09-20

### Added

- **Interactive interface.** `gitscope tui [PATH_OR_URL]`, built with `ratatui`: tabs for the overview, contributors, files, activity and commits; selectable lists; a commit list filtered as you type (author, email, message or hash) that shows the files, insertions and deletions of the selected commit; `Enter` on a contributor opens their commits; a help overlay (`?`). It accepts the usual filters and URLs.
- **Configurations tab** (key `6`) to change the interface language, **English** or **Português (Brasil)**: the screen is translated at once and the choice is saved in a small JSON file in the user's configuration directory (`%APPDATA%\gitscope\config.json` on Windows, `~/Library/Application Support/gitscope/config.json` on macOS, `$XDG_CONFIG_HOME/gitscope/config.json` or `~/.config/gitscope/config.json` on Linux; `GITSCOPE_CONFIG` overrides it). An unreadable file is ignored with a warning. The texts live in one complete struct per language, so a missing translation is a compile error.
- Long messages, authors and file paths wrap inside the commit details instead of running off the screen (by words, measured in screen columns so accents and wide characters count right; the continuation lines stay aligned under the value).
- Tests for the interface: every key against the state (no terminal needed) and every screen drawn on ratatui's in-memory `TestBackend`, including tiny sizes and empty repositories.

### Changed

- The bar, sparkline and truncation helpers moved from the terminal renderer to the shared `output::format` module (the output is unchanged).

### Known limitations

- It needs a real terminal and refuses to run on pipes. The analysis runs before the interface opens (no progress screen), and the detail of every commit is kept in memory.
- Only the interface is translated; the report, the messages and the other commands stay in English.
- No mouse support. The automated tests never drive a real terminal: they check the state and the drawing, not the terminal setup and restore.

## [0.7.0] - 2026-09-20

### Added

- **Markdown templates.** Write `{variables}` in your own README and GitScope fills them in, changing nothing else:
  - `gitscope update [FILE]` refreshes the `<!-- gitscope:start -->` / `<!-- gitscope:end -->` blocks of a file in place (default: `README.md` at the repository root). Each block keeps its template in an invisible HTML comment so it can be refreshed again; repeating an update with the same data is a no-op.
  - `gitscope markdown FILE [-o OUT]` prints a template with the variables resolved and never modifies it; a file without blocks is resolved as a whole.
  - `gitscope variables` lists the variables.
- Variables: `{repository_name}`, `{first_commit}`, `{last_commit}`, `{period}`, `{branches}`, `{tags}`, `{commits}`, `{first_commit_message}`, `{last_commit_message}`, `{most_active_hour}`, `{contributors}`, `{top_author}`, `{top_author_commits}`, `{top_author_percentage}`, `{files}`, `{insertions}`, `{deletions}`, `{language_count}`, `{top_language}` and the compound `{author:NAME:commits|insertions|deletions}`. They live in a central registry: adding one is a single entry.
- Safety: the file is validated (blocks, variables) and rendered in memory before anything is written; on any error it is left untouched; writes are atomic and keep permissions; unchanged files are not rewritten; line endings, BOM and the final newline are preserved. Unknown variables fail with the list of available ones and a "did you mean"; `--keep-unknown` turns them into warnings.
- JSON: `commits` gains `first_message`, `last_message`, `insertions` and `deletions` (additive; `schema_version` stays 1).
- Documentation: [docs/MARKDOWN-EN.md](docs/MARKDOWN-EN.md). All documentation now exists in **Portuguese (primary)** and **English** (`-EN` files), with a test (`tests/docs.rs`) that checks the links and structure of both languages.

### Changed

- **The project is now called GitScope** (it was GitStats, and briefly GitStatus, a name already taken on crates.io). The crate, library and binary are `gitscope`, the terminal header reads `GITSCOPE`, the error type is `GitScopeError`. The block markers are `gitscope:start` / `gitscope:end`; the former `gitstatus:` and `gitstats:` markers, and the template comment an older version wrote, are still accepted and migrated on the next `update`.
- Report options cannot be combined with `update`, `markdown` or `variables`. A subcommand name is only special as the first argument, so use `./update` for a directory with that name.

### Known limitations

- A block whose template contains `-->` cannot be stored by `update` (it would end the HTML comment that keeps it); use `markdown -o` with a separate template file.
- Values are inserted as they are: a `|` in a commit message breaks the table cell it is put in.

## [0.6.0] - 2026-09-20

### Added

- **Remote repositories.** `gitscope <URL>` clones the repository into a temporary directory, analyses it and deletes the clone. Supported: `https://`, `http://`, `git://`, `file://`. All options and filters work on remote repositories.
- The clone is bare (no working tree), complete, and cleaned up on success and on failure. Progress is shown on stderr when it is a terminal.
- For clones, *branches* counts the remote's branches and `--branch` accepts remote branches by plain name or as `origin/<name>`.
- Credentials embedded in URLs are redacted from every message.
- Cargo feature `remote` (default) enables HTTPS through `git2/https`; `--no-default-features` builds a local-only binary without TLS/OpenSSL.
- Library: `open_source`, `analyze_repository`, `is_remote`, `Repository::clone_to_temp`, `Repository::close`, module `remote`.

### Changed

- Cleaner clone error messages, with a hint when the host answers 401/403 (private or non-existent repository).
- `tempfile` is now a regular dependency.

### Known limitations

- Only unauthenticated transports: SSH and private repositories are not supported.
- If the process is killed (for example with Ctrl-C) during or after a clone, the `gitscope-*` temporary directory can be left behind.

## [0.5.0] - 2026-09-20

First public version: the complete MVP (roadmap v0.1 to v0.5).

### Added

- Analysis of local repositories through libgit2 (`git2`), from a working directory, a `.git` directory or a bare repository. The `git` executable is not required.
- Repository facts: name, local branches, tags.
- Commit statistics: total, first and last commit, commits per weekday and hour, most active hour.
- Contributors: commits, percentage, files changed, insertions, deletions, first and last commit. Identity is the lowercase email.
- File statistics: files in the snapshot, extensions, most modified files. Detection of added, modified, deleted and renamed files.
- Language breakdown from a table-driven extension mapping.
- Activity per month, with quiet months filled with zero.
- Filters: `--since`, `--until`, `--author` (name or email, case-insensitive), `--branch`. Filters combine.
- Author view (`--author`) with a per-contributor summary.
- `--all`: every commit matching the filters, with author, date, message, changed files, insertions and deletions. `--verbose` prints the full block for each commit.
- `--verbose` diagnostics on stderr; `--top` to limit ranked lists in the terminal output.
- `--json` export with a versioned, documented schema (`schema_version` 1).
- Formatted terminal output with sections, aligned tables, bars and a sparkline.
- Documentation (README, JSON schema, architecture, contributing), dual MIT/Apache-2.0 license and a GitHub Actions workflow (format, clippy, tests on Linux, Windows and macOS, MSRV, release build).

### Known limitations

- Line statistics are computed with libgit2, which is slower than `git` on very large files (see *Performance* in the README).
- `.mailmap` is not applied; the same person using two emails counts as two contributors.
- Merge commits are counted but contribute no files or lines.
- Copies are not detected and history is not followed across renames.
- Branches counted are local branches only.
