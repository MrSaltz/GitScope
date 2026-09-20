# gitscope

[Português](README.md) · **English**

**A Rust CLI for analyzing Git repository statistics.**

Point `gitscope` at a local repository and get commits, contributors, files,
languages and activity, as a readable terminal report or as JSON.

- Reads the repository directly with [`git2`](https://crates.io/crates/git2) (libgit2). **No `git` executable is needed.**
- Works on a working directory, a `.git` directory or a bare repository.
- Filter by author, date range and branch; filters combine.
- Inspect every commit of an author, with the files each one touched and its line counts.
- Stable, documented JSON output for scripts and dashboards.
- Put the numbers in your own README: write `{commits}`, `{contributors}`... in Markdown and let `gitscope update` fill them in.
- Analyse a **remote repository** straight from its URL: it is cloned into a temporary directory that is deleted afterwards.
- Read-only and local by default: the network is used only when you pass a URL. No server, database, accounts or telemetry.

## Example

```console
$ gitscope ./my-project --top 5
```

```text
╭──────────────────────────────────────────────────────────╮
│                     GITSCOPE v1.0.0                      │
│                        my-project                        │
╰──────────────────────────────────────────────────────────╯

Repository
────────────────────────────────────────────────────────────
  Commits                                  102
  Contributors                               3
  Branches                                   3
  Tags                                       2
  Period               2026-01-02 → 2026-04-26

Commits
────────────────────────────────────────────────────────────
  Total                                    102
  First commit                      2026-01-02
  Last commit                       2026-04-26
  Most active hour                       17:00

  By weekday
  Mon  █████████████████████████████  19
  Tue  ██████████████                 9
  Wed  █████████████████████          14
  Thu  ███████████████████████        15
  Fri  ████████████████████           13
  Sat  ██████████████████             12
  Sun  ██████████████████████████████ 20

  By hour (UTC)
  ▂▄▂······▇▄▄▄▅▆▅██▆▁▂▁▂▁
  0     6     12    18

Top contributors
────────────────────────────────────────────────────────────
  Alice                56 commits  54.9%      +770      -72
  Bob                  35 commits  34.3%      +457      -65
  Charlie              11 commits  10.8%      +141       -4

Files
────────────────────────────────────────────────────────────
  Total files                               10

  Extensions
  .rs               4
  .md               1
  .py               1
  .sh               1
  .toml             1
  … and 2 more (raise --top to see them)

  Most modified files
  Cargo.toml                                           15
  src/lib.rs                                           13
  tools/report.py                                      12
  src/stats.rs                                         11
  web/app.ts                                           11
  … and 5 more (raise --top to see them)

Languages
────────────────────────────────────────────────────────────
  Rust            50.0%  ██████████
  TypeScript      25.0%  █████
  Python          12.5%  ███
  Shell           12.5%  ███

Activity
────────────────────────────────────────────────────────────
  2026-01  █████████████████              19
  2026-02  ███████████████████            21
  2026-03  ██████████████████████████     29
  2026-04  ██████████████████████████████ 33
```

*(Real output. Rendering needs a terminal with UTF-8 support.)*

## Installation

### Pre-built binaries

Every release on GitHub (the *Releases* page) has one archive per platform and a `SHA256SUMS` file:

| Platform | Archive |
|---|---|
| Linux (x86_64) | `gitscope-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |
| Windows (x86_64) | `gitscope-vX.Y.Z-x86_64-pc-windows-msvc.zip` |
| macOS (Apple Silicon) | `gitscope-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `gitscope-vX.Y.Z-x86_64-apple-darwin.tar.gz` |

Extract it, put `gitscope` (`gitscope.exe` on Windows) in a folder that is in your `PATH` and check with `gitscope --version`. To verify the download:

```bash
sha256sum -c SHA256SUMS --ignore-missing                                       # Linux
shasum -a 256 -c SHA256SUMS --ignore-missing                                   # macOS
Get-FileHash .\gitscope-vX.Y.Z-x86_64-pc-windows-msvc.zip -Algorithm SHA256    # Windows (PowerShell): compare with SHA256SUMS
```

The Linux and macOS binaries carry OpenSSL inside, so they do not depend on the system's libssl. The binaries appear here once the first release is published; until then, build from source.

### From source

Requirements:

- [Rust](https://rustup.rs) 1.87 or newer.
- A C compiler, because `git2` builds libgit2 from source: the *MSVC Build Tools* on Windows, `gcc`/`clang` on Linux, Xcode Command Line Tools on macOS.
- On **Linux** and **macOS**, OpenSSL and `pkg-config` (for HTTPS clones): `sudo apt install pkg-config libssl-dev` on Debian/Ubuntu, `sudo dnf install pkgconf-pkg-config openssl-devel` on Fedora, `brew install openssl pkg-config` on macOS. Windows uses the system TLS stack and needs nothing extra.

No OpenSSL, or no need for remote repositories? Build a local-only binary with `cargo install --path . --no-default-features` (or bundle OpenSSL with `--features vendored-openssl`, which needs `perl` and `make`).

```bash
git clone https://github.com/MrSaltz/GitScope.git
cd gitscope
cargo install --path .
```

Or, without cloning:

```bash
cargo install --git https://github.com/MrSaltz/GitScope.git
```

Check the installation:

```bash
gitscope --version
```

### Build without installing

```bash
cargo build --release
./target/release/gitscope --help      # target\release\gitscope.exe on Windows
```

## Usage

```text
gitscope [OPTIONS] [PATH_OR_URL]      print the report (default)
gitscope update [FILE]                update the {variables} of a README in place
gitscope markdown FILE                print a Markdown template with the variables resolved
gitscope variables                    list the variables
gitscope tui [PATH_OR_URL]            browse the statistics in an interactive interface
```

`PATH_OR_URL` is the repository to analyse and defaults to the current directory. It can be a working directory, a `.git` directory, a bare repository, or the URL of a [remote repository](#remote-repositories). Parent directories are not searched.

```bash
gitscope                         # analyse the current directory
gitscope /path/to/repository     # analyse another repository
gitscope ./repo/.git             # the .git directory works too
gitscope https://github.com/owner/repo.git   # a remote repository
```

### Options

| Option | Description |
|---|---|
| `--since <DATE>` | Only commits authored on or after this day (`YYYY-MM-DD`, inclusive). |
| `--until <DATE>` | Only commits authored on or before this day (`YYYY-MM-DD`, inclusive). |
| `--author <AUTHOR>` | Only commits whose author **name or email contains** this text (case-insensitive). Switches to the [author view](#author-analysis). |
| `--branch <BRANCH>` | Analyse this branch instead of `HEAD`. Local branches are tried first, then remote-tracking ones such as `origin/main`. |
| `--json` | Print the report as JSON instead of formatted text. See [JSON output](#json-output). |
| `--all` | List **every commit** matching the filters, with details. Per-commit details are collected only when this flag is present. |
| `-v`, `--verbose` | Print diagnostics to **stderr**. Combined with `--all`, print the full detail block of each commit. |
| `--top <N>` | Maximum rows in the ranked lists of the terminal output (default `10`). JSON is never truncated. |
| `-h`, `--help` / `-V`, `--version` | Help and version. |

Filters combine: a commit must satisfy all of them.

### Exit codes and streams

| Code | Meaning |
|---|---|
| `0` | Success (including an empty repository or filters that match nothing). |
| `1` | Runtime error: path not found, not a Git repository, unknown branch, `--since` after `--until`, unsupported URL, failed clone, libgit2 failure. |
| `2` | Invalid command line (for example a malformed date). |

The report goes to **stdout**; errors and `--verbose` diagnostics go to **stderr**, so `gitscope ./repo --json > stats.json` always produces a clean file.

## Examples

Filter by date range:

```bash
gitscope ./repo --since 2026-01-01 --until 2026-06-30
```

Analyse a branch:

```bash
gitscope ./repo --branch main
```

Everything at once:

```bash
gitscope ./repo \
  --author "Widison" \
  --branch main \
  --since 2026-01-01 \
  --until 2026-09-01 \
  --all
```

Pipe it into a pager or export it:

```bash
gitscope ./repo | less
gitscope ./repo --json > stats.json
```

### Author analysis

```bash
gitscope ./my-project --author Alice --top 3
```

```text
╭──────────────────────────────────────────────────────────╮
│                     GITSCOPE v1.0.0                      │
│                        my-project                        │
╰──────────────────────────────────────────────────────────╯

Filters: author "Alice"

Contributor
────────────────────────────────────────────────────────────
  Name                                   Alice
  Email                      alice@example.com
  Commits                                   56
  Files changed                             56
  Insertions                              +770
  Deletions                                -72
  First commit                      2026-01-02
  Last commit                       2026-04-26
...
```

If the text matches several identities (for example two email addresses), one *Contributor* block is printed for each.

### Inspecting an author's commits (`--all`)

```bash
gitscope ./my-project --author Bob --all --since 2026-04-20
```

```text
Commits by Bob
────────────────────────────────────────────────────────────

2026-04-26  09:38  ae00d6a
fix: handle empty repositories
```

Add `--verbose` to get the full block for every commit, including each file that was touched (`+` added, `M` modified, `-` deleted, `R` renamed):

```bash
gitscope ./my-project --author Bob --all --verbose --since 2026-04-24
```

```text
Commits by Bob
────────────────────────────────────────────────────────────

ae00d6a
────────────────────────────────────────
Author       Bob <bob@example.com>
Date         2026-04-26 09:38
Message      fix: handle empty repositories

Changes
  M src/main.rs

Files changed      1
Insertions       +10
Deletions         -0
```

`--all` also works without `--author`: it then lists every commit that passes the other filters.

## Remote repositories

Pass a URL instead of a path and `gitscope` clones the repository into a temporary directory, analyses it and deletes the clone. Every option and filter works as usual:

```bash
gitscope https://github.com/owner/repo.git
gitscope https://github.com/owner/repo.git --since 2026-01-01 --author alice --all
gitscope https://github.com/owner/repo.git --branch develop --json > stats.json
```

While it works, stderr shows:

```text
gitscope: cloning https://github.com/owner/repo.git into a temporary directory...
  Receiving objects:  45% (1234/2741), 3.2 MiB      <- live progress, only on a terminal
```

How it behaves:

- **Supported URLs:** `https://`, `http://`, `git://` and `file://`. An existing local path always wins over URL syntax.
- **Public repositories only.** SSH URLs (`ssh://…`, `git@host:owner/repo`) are rejected with an explanation, because authentication is out of scope. A private repository, or one that does not exist (hosts answer `401` for both), fails with a message saying so; there are no credential prompts.
- **Credentials are never printed.** If you put a token in a URL (`https://token@host/…`), it is replaced with `***` in every message.
- **Complete history.** Statistics need every commit, so the whole history is downloaded: expect a few seconds to minutes for large repositories. The clone is *bare*: no working tree, no files checked out, no hooks run.
- **Clean-up.** The temporary directory (`gitscope-XXXXXX` inside your system temp directory) is removed when the analysis ends, also when it fails. The clone message, progress and diagnostics go to **stderr**, so `--json` on stdout stays clean. If the process is killed (for example with Ctrl-C) the operating system does not let it clean up, and the `gitscope-*` directory may stay behind; it is safe to delete.
- **Branches.** For a clone, *branches* counts the branches of the remote, and `--branch` accepts a remote branch by its plain name (`--branch develop`) or as `origin/develop`.
- **Validation first.** Bad arguments (for example `--since` after `--until`) are reported before anything is downloaded.

## Markdown templates

Put statistics in your own README. Write the Markdown you want and drop **variables** where the numbers belong:

```markdown
## Statistics

<!-- gitscope:start -->

Project started: {first_commit}

Latest commit: {last_commit}

Total commits: {commits}

Contributors: {contributors}

<!-- gitscope:end -->
```

```bash
gitscope update
```

```markdown
<!-- gitscope:start -->
<!-- gitscope:template

Project started: {first_commit}
...
-->

Project started: 2026-01-02

Latest commit: 2026-04-26

Total commits: 102

Contributors: 3

<!-- gitscope:end -->
```

GitScope replaces **only the variables**, and only inside the blocks: everything else in the README is left byte for byte as you wrote it. It imposes no layout, so a sentence, a table, a quote, a badge or HTML all work:

```markdown
🚀 This project has {commits} commits from {contributors} contributors.

| Metric | Value |
|---|---:|
| Commits | {commits} |
| Files | {files} |
```

| Command | What it does |
|---|---|
| `gitscope update [FILE]` | Refreshes the blocks of `FILE` (default `README.md` at the repository root), in place and atomically. |
| `gitscope markdown FILE` | Prints `FILE` with the variables resolved; never modifies it (`-o OUT` writes another file). |
| `gitscope variables` | Lists the variables: `{commits}`, `{contributors}`, `{first_commit}`, `{top_author}`, `{author:NAME:commits}`... |

The invisible `<!-- gitscope:template -->` comment is how the block remembers its template, so the next `gitscope update` can refresh the numbers (repeating it with the same data changes nothing). Unknown variables are an error that lists the available ones; a malformed block or any error leaves the file untouched.

Full guide, variable list, syntax, safety guarantees and examples: **[docs/MARKDOWN-EN.md](docs/MARKDOWN-EN.md)**.

## Interactive interface (TUI)

```bash
gitscope tui                                   # the current directory
gitscope tui ./repo --since 2026-01-01
gitscope tui https://github.com/owner/repo.git
```

Browse the same statistics with the keyboard, in six tabs: **Overview**, **Contributors**, **Files**, **Activity**, **Commits** and **Configurations**. The repository is analysed once, before the interface opens (the usual `--since`, `--until`, `--author` and `--branch` filters and URLs work), and the interface only browses the result. The commit list can be filtered as you type, by author, email, message or hash, and shows the files, insertions and deletions of the selected commit.

| Key | Action |
|---|---|
| `←` `→`, `Tab`, `h` `l`, `1`…`6` | Switch tab |
| `↑` `↓`, `k` `j`, `PgUp` `PgDn`, `Home` `End` | Move the selection |
| `Enter` (`Space` too) | On *Contributors*: open that person's commits; on *Configurations*: change the setting |
| `/` | Filter the commits; `Enter` keeps the filter, `Esc` clears it |
| `?` | Help |
| `q`, `Ctrl+C` | Quit |

It needs a real terminal (stdin and stdout); for pipes and files use the plain report or `--json`. The commit list keeps the detail of every commit in memory, so on histories with hundreds of thousands of commits it is heavier than the plain report. The interface comes in English and Portuguese (see below).

### Configuration

The **Configurations** tab (key `6`) changes the interface language: **English** or **Português (Brasil)**. Select the setting and press `Enter` or `Space`; the screen is translated at once and the choice is saved for the next time in a small JSON file:

```json
{ "language": "pt" }
```

| System | Configuration file |
|---|---|
| Windows | `%APPDATA%\gitscope\config.json` |
| macOS | `~/Library/Application Support/gitscope/config.json` |
| Linux | `$XDG_CONFIG_HOME/gitscope/config.json` (or `~/.config/gitscope/config.json`) |

Set `GITSCOPE_CONFIG` to use another file. Only the interface is translated: the report, the messages and the other commands stay in English. An unreadable configuration file is ignored with a warning and rewritten the next time you change a setting.

## JSON output

```bash
gitscope ./my-project --json > stats.json
```

```json
{
  "schema_version": 1,
  "repository": { "name": "my-project", "branches": 3, "tags": 2 },
  "filters": { "since": null, "until": null, "author": null, "branch": null },
  "commits": {
    "total": 102,
    "first": "2026-01-02T11:50:00Z",
    "last": "2026-04-26T14:44:00Z",
    "most_active_hour": 17,
    "by_weekday": [ { "weekday": "Mon", "commits": 19 }, "..." ],
    "by_hour": [ { "hour": 0, "commits": 2 }, "..." ]
  },
  "contributors": [
    {
      "name": "Alice",
      "email": "alice@example.com",
      "commits": 56,
      "percentage": 54.9,
      "files_changed": 56,
      "insertions": 770,
      "deletions": 72,
      "first_commit": "2026-01-02T11:50:00Z",
      "last_commit": "2026-04-26T14:44:00Z"
    }
  ],
  "files": {
    "total": 10,
    "extensions": [ { "extension": "rs", "count": 4 }, "..." ],
    "most_modified": [ { "path": "Cargo.toml", "modifications": 15 }, "..." ]
  },
  "languages": [ { "name": "Rust", "files": 4, "percentage": 50.0 }, "..." ],
  "activity": { "by_month": [ { "month": "2026-01", "commits": 19 }, "..." ] }
}
```

*(Arrays abbreviated with `"..."` for this README.)*

With `--all`, a `commit_details` array is added, newest commit first:

```bash
gitscope ./my-project --author Bob --all --json --since 2026-04-24
```

```json
"commit_details": [
  {
    "hash": "ae00d6a031eff59a79fcc1de177d57d25680329c",
    "author": "Bob",
    "email": "bob@example.com",
    "date": "2026-04-26T09:38:00Z",
    "message": "fix: handle empty repositories",
    "files_changed": 1,
    "insertions": 10,
    "deletions": 0,
    "changes": [ { "path": "src/main.rs", "kind": "modified" } ]
  }
]
```

The schema is versioned and documented field by field in **[docs/JSON-EN.md](docs/JSON-EN.md)**, including the stability policy and `jq` recipes.

## How the numbers are defined

Precise definitions matter for a statistics tool, so here they are:

- **Analysed history.** The commits reachable from `HEAD`, or from `--branch`. Commits that only exist on unmerged branches are not included. Merge commits are counted as commits.
- **Dates.** Every commit is placed by its **author date, in UTC**. Weekdays, hours and months are UTC buckets. `--since` starts at 00:00:00 of that day and `--until` runs through 23:59:59 of that day.
- **Contributors.** An author is identified by their **email address** (case-insensitive), so `Bob` and `Bob Smith` using the same email are one contributor. The name shown is the one from their most recent commit. `.mailmap` is not applied.
- **Percentages.** A contributor's share of the commits that passed the filters (the terminal shows one decimal, JSON two).
- **Modified file.** A file is *modified* once for every commit in which it appears as changed against the first parent: added, modified, deleted or the destination of a rename. A file touched in two commits has two modifications. Renames are detected (50 % similarity) and counted for the new path; history is not followed across renames, and copies are not detected.
- **Insertions and deletions.** Added and removed lines of each commit against its first parent. The first commit is diffed against an empty tree. **Merge commits contribute no files or lines**, because their changes are already attributed to the commits that were merged. Binary files count as a changed file with zero lines.
- **Files, extensions and languages** describe a **snapshot**: the file tree at the tip of the analysed history (`HEAD` or `--branch`). They ignore `--since`, `--until` and `--author`. *Most modified files* and everything else are computed from the filtered history.
- **Extensions.** The last extension, lowercased (`archive.tar.gz` → `.gz`). Files such as `Makefile` and dotfiles such as `.gitignore` have no extension and appear as `(none)`.
- **Languages.** Derived from the extension using the table in [`src/analysis/languages.rs`](src/analysis/languages.rs). Percentages are by **number of files** and only consider files with a recognised language: prose and data files (Markdown, JSON, TOML…) do not dilute the chart.
- **Branches.** Local branches (for a remote URL: the branches of the remote). **Tags** include lightweight and annotated tags.
- **Empty results** are not errors: a repository without commits, or an author/date filter that matches nothing, prints a message (or valid JSON with `"total": 0`) and exits with `0`.

## Performance

`gitscope` processes history **incrementally**: one pass over the commits, one diff at a time, nothing kept per commit unless `--all` is used. Author and date filters are applied *before* diffing, so `--author` and `--since` runs skip the expensive part for every commit they exclude.

Measured with a release build on a 361-commit repository (Windows, 12 cores):

| Run | Time |
|---|---|
| Walk only (filter matches no commit) | 0.06 s |
| Full report | 3.9 s |
| `git log --numstat` for comparison | 1.0 s |

Virtually all of the time is spent computing line counts with libgit2. This repository is close to a worst case: it has two ~1 MB files that were each edited in nearly 300 commits, so hundreds of megabytes of text have to be diffed. Rename detection costs almost nothing, and results match `git` (commit counts, per-author counts, insertions, deletions, file counts and most-modified files were cross-checked). Parallel diffing is the first candidate if this ever needs to be faster; see the [roadmap](#roadmap).

### Benchmarks

```bash
cargo bench                                # a repository with 1000 commits, 7 runs each
GITSCOPE_BENCH_COMMITS=5000 cargo bench    # a bigger one
```

The benchmark builds a synthetic repository (20 files, 4 authors) and times each stage. Best times on a Windows machine with 12 cores, for 1000 commits:

| Scenario | Best time |
|---|---|
| Walk only (no commit matches, no diffs) | 95 ms |
| Full report | 2.2 s |
| Full report with per-commit details (`--all`) | 2.2 s |
| One author (diffs only their commits) | 0.75 s |
| Render the terminal report | 0.4 ms |
| Render the JSON (with per-commit details) | 0.4 ms |
| Render a Markdown template | < 0.01 ms |

The numbers depend on the machine, so use them to compare versions with each other. The synthetic repository has loose (unpacked) objects, which makes every diff read more files than in a packed repository. What carries over is the shape: walking is cheap, the diffs are the cost, `--all` adds almost nothing and rendering is free.

## How it works

```
Git repository ─► repository.rs ─► CommitRecord ─► analysis/* ─► RepositoryStats ─┬─► terminal
                  (git2 only)      (domain model)  (pure)                          └─► JSON
```

The layers are strictly separated: only `repository.rs` talks to `git2`, `analysis/` never prints, and `output/` (terminal, JSON and Markdown) never recalculates. See **[docs/ARCHITECTURE-EN.md](docs/ARCHITECTURE-EN.md)** for details and for how to extend the tool.

The crate is both a binary (`gitscope`) and a library (`gitscope::analyze_path`), which is what the integration tests use.

## Development

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

Integration tests build small, deterministic repositories with `git2` in temporary directories (fixed authors and timestamps), so they need neither the `git` executable nor network access. Remote analysis is tested by cloning `file://` URLs and by failing against a closed local port. They cover empty repositories, invalid paths, several authors, different months and hours, every kind of file change, merges, filters, JSON and `--all`.

What stays stable between versions is described in **[docs/STABILITY-EN.md](docs/STABILITY-EN.md)**; how a release is made, in **[docs/RELEASING-EN.md](docs/RELEASING-EN.md)**.

## Roadmap

- [x] **v0.1 — Core:** open a repository, commits, branches, tags, contributors, first/last commit, data model
- [x] **v0.2 — Statistics:** commits per author / month / weekday / hour, files, extensions, languages, activity
- [x] **v0.3 — CLI:** `--since`, `--until`, `--author`, `--branch`, `--verbose`, formatted output
- [x] **v0.4 — Export:** `--json` with a stable, documented schema
- [x] **v0.5 — Commit inspection:** `--all`, per-author history, files, insertions and deletions per commit
- [x] **v0.6 — Remote repositories:** analyse a URL through a temporary clone that is cleaned up afterwards
- [x] **v0.7 — Markdown templates:** `{variables}` in your own README, `gitscope update` / `markdown` / `variables`, blocks, safe atomic writes
- [x] **v0.8 — TUI:** `gitscope tui`, an interactive interface with [`ratatui`](https://ratatui.rs): six tabs, a commit list filtered as you type, details of each commit, language selection (English / Portuguese)
- [x] **v1.0 — Stable:** a stability policy, basic benchmarks (`cargo bench`), a release workflow that builds binaries for four platforms (still to be run for the first time) and installation docs

Ideas under consideration (not commitments, in no particular order):

- More compound variables: `{commit:first}`, `{commit:last}`, `{language:Rust:files}`, more `{author:NAME:FIELD}` fields (`files_changed`, `percentage`, first/last commit).
- `gitscope update --check`, which fails when the README is out of date (for CI), and a GitHub Action that keeps it fresh.
- `.mailmap` support, so one person with several emails counts once.
- Parallel diffing, to speed up large histories (see [Performance](#performance)).

## Contributing

Contributions are welcome. Please read **[CONTRIBUTING-EN.md](CONTRIBUTING-EN.md)** first; it explains the project scope (a local CLI, deliberately small), how to run the checks and how to add a language.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option. See [LICENSE](LICENSE).
