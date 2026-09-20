# Stability policy

[Português](STABILITY.md) · **English**

From version 1.0, GitScope follows [Semantic Versioning](https://semver.org/) for what this page lists as stable. It says exactly what that covers, so you know what you can build scripts, CI jobs and READMEs on.

## What the version number promises

| Area | Guarantee |
|---|---|
| Commands, options and what they do | Stable. New commands and options can appear in minor versions; removing or renaming one, or changing what it means, is a major change. |
| Exit codes (`0`, `1`, `2`) and the stdout/stderr split | Stable. |
| JSON output | Stable, versioned by `schema_version` (the policy is in [JSON-EN.md](JSON-EN.md)). |
| Markdown templates: markers, `{variable}` syntax, escaping and the documented variables | Stable. New variables can appear in minor versions; removing one or changing the meaning of its value is a major change. |
| Configuration file | Stable: the `language` key. New keys may appear; unknown keys are ignored. |

## What it does not promise

| Area | Why |
|---|---|
| The text of the terminal report | It is for people: labels, columns and bars may change in any release. For scripts, use `--json`. |
| The interactive interface (`gitscope tui`) | Layout, keys and texts may change in minor versions; the documented keys are kept whenever possible. |
| The text of error messages | Only that they go to stderr with exit code 1 is promised. |
| The internals of the Rust library | See below. |

## The Rust library

The crate is both a binary and a library. These entry points and types follow Semantic Versioning from 1.0: `analyze_path`, `analyze_repository`, `open_source`, `is_remote`, `analysis::Options`, the types in `model` (their fields mirror the JSON schema), `GitScopeError` (`non_exhaustive`, so new variants do not break its users) and `config::{Config, Language}`.

Everything else that is `pub` (the analysis accumulators, the renderers, the interface, `repository`, `remote` and the details of `output::markdown` and `output::variables`) exists for the tests and the benchmarks and may change in any release. Do not depend on it.

## Changing something that is stable

A breaking change bumps the major version and shows up in the [CHANGELOG](../CHANGELOG-EN.md), under *Changed* or *Removed*. Whenever possible, the old behaviour is kept for one minor version with a warning on stderr before it is removed.
