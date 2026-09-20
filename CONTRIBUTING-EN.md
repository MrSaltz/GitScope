# Contributing to gitscope

[Português](CONTRIBUTING.md) · **English**

Thanks for your interest! GitScope is intentionally small: **a local CLI that analyses Git repositories.** Please keep contributions within that scope.

## Scope

In scope: better or new statistics, correctness fixes, output polish, performance work backed by measurements, documentation, tests, and the items on the [roadmap](README-EN.md#roadmap).

Out of scope (please do not open PRs for these): web servers or dashboards, databases, authentication or accounts (this includes SSH and credential prompts for remote clones), cloud sync, telemetry, AI features, or anything that collects external data. If you are unsure, open an issue first.

## Getting started

Requirements: Rust 1.87+ and a C compiler (see the [README](README-EN.md#installation)). The `git` executable is **not** required, not even to run the tests, and the tests never use the network. On Linux you also need `pkg-config` and the OpenSSL development files (or use `--no-default-features`, see the README).

```bash
git clone https://github.com/MrSaltz/GitScope.git
cd gitscope
cargo test
```

Before sending a change, make sure all of these pass (CI runs the same checks):

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --no-default-features   # the local-only build must keep working
```

## Project rules

Read [docs/ARCHITECTURE-EN.md](docs/ARCHITECTURE-EN.md). The short version:

- Only `src/repository.rs` uses `git2`.
- `src/analysis/` is pure: no printing, no Git access.
- `src/output/` only formats; it never recomputes statistics.
- Never shell out to `git`.
- Never print a URL without going through `remote::redact`: it may contain credentials.
- Process history incrementally; do not keep diffs or per-commit data in memory unless a feature needs it.
- Prefer simplicity: no new dependency, abstraction or flag without a clear need.

## Comments in the code

Comment sparingly: only what the code does not make clear, such as a non-obvious rule, a pitfall or the *why* of a choice, never the *what* it already says. Each comment is at most one or two lines and is written in **Portuguese**. The exceptions are the `///` comments in `src/cli.rs`: clap turns them into the `--help` text, so they are user-facing and stay in English, like every other message the tool prints and the documentation.

## Bilingual documentation

**Portuguese is the primary language**; every `.md` file has an English twin with the `-EN` suffix:

| Portuguese (primary) | English |
|---|---|
| `README.md` | `README-EN.md` |
| `CONTRIBUTING.md` | `CONTRIBUTING-EN.md` |
| `CHANGELOG.md` | `CHANGELOG-EN.md` |
| `docs/ARCHITECTURE.md` | `docs/ARCHITECTURE-EN.md` |
| `docs/JSON.md` | `docs/JSON-EN.md` |
| `docs/MARKDOWN.md` | `docs/MARKDOWN-EN.md` |
| `docs/STABILITY.md` | `docs/STABILITY-EN.md` |
| `docs/RELEASING.md` | `docs/RELEASING-EN.md` |

When you change one, **update the other in the same PR**. The `tests/docs.rs` test helps: it fails if a link or anchor is broken and if the two languages of a document have different structures (number of headings, code blocks, tables and links). It does not check the translation itself; that is up to you. The links of each language point to documents of the same language. Program output examples are the same in both, because the tool prints in English.

## Tests

Every behaviour change needs a test.

- **Unit tests** live next to the code. For statistics, build `CommitRecord`s with `analysis::testutil`.
- **Integration tests** in `tests/` build real repositories with the `TestRepo` helper in `tests/common/mod.rs`:

  ```rust
  let t = TestRepo::new();
  t.by("Alice", "alice@example.com")
      .at("2026-01-03 12:30")
      .message("feat: add statistics")
      .write("src/main.rs", "fn main() {}\n")
      .commit();
  ```

  Use fixed dates so results are deterministic.
- **The JSON schema is pinned** by `json_schema_is_stable` in `tests/cli.rs`. If you change it on purpose, update [docs/JSON-EN.md](docs/JSON-EN.md) and the changelog, and read the stability policy there first.

## Adding a Markdown variable

Add one entry to `standard_variables()` in `src/output/variables.rs` (name, group, description, resolver), a test next to the others in that file, and a row in [docs/MARKDOWN-EN.md](docs/MARKDOWN-EN.md) (and in `docs/MARKDOWN.md`): a test fails when a registered variable is not documented. Resolvers only read `RepositoryStats`; if a value needs new data, compute it in `analysis/` and document it in [docs/JSON-EN.md](docs/JSON-EN.md).

## Adding a language

Add one `(extension, language)` line to `LANGUAGES` in `src/analysis/languages.rs`. Extensions are lowercase, without the dot, and must be unique.

## Commits and pull requests

- Keep pull requests focused; one topic per PR.
- Use clear, imperative commit messages. [Conventional Commits](https://www.conventionalcommits.org) (`feat:`, `fix:`, `docs:`, `test:`, `refactor:`) are appreciated but not enforced.
- Update `README.md` / `docs/` and `CHANGELOG.md` (and their `-EN` twins) when behaviour changes.
- Performance claims need numbers: describe the repository, the command and the before/after timings.

## Reporting bugs

Please include the `gitscope --version`, your OS, the exact command, and, if possible, a minimal repository (or the sequence of commits) that reproduces the problem. `--verbose` output helps.

## License

By contributing you agree that your work is dual-licensed under **MIT OR Apache-2.0**, the same as the project (see [LICENSE](LICENSE)).
