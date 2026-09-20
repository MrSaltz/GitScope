# Releasing

[Português](RELEASING.md) · **English**

This guide is for whoever maintains the project. Publishing uses the `.github/workflows/release.yml` workflow: when a `vX.Y.Z` tag is pushed, it builds the binaries, creates the GitHub Release and attaches the archives and `SHA256SUMS`.

## Before the first release (one time)

1. Push the code to the GitHub repository (`repository` and `homepage` are already in `Cargo.toml`).
2. Check under *Settings → Actions → General* that workflows may write to the repository (the workflow already asks for `contents: write`; organizations can restrict it).
3. On crates.io, create the account and an API token and run `cargo login`. Check again that the name is free with `cargo search gitscope`.

## Preparing the release

1. Write the `## [x.y.z] - date` section in `CHANGELOG.md` and `CHANGELOG-EN.md`.
2. Change `version` in `Cargo.toml`. `cargo build` updates `Cargo.lock`. The report header shows the version, so search the READMEs for `GITSCOPE v` and regenerate the examples if it appears.
3. Run the checks: `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo test --no-default-features` and `cargo publish --dry-run`.
4. Commit, merge to `main` and wait for CI to turn green.

## Publishing

```bash
git tag v1.0.0
git push origin v1.0.0
```

The workflow checks that the tag equals the `Cargo.toml` version, builds four binaries (Linux x86_64, Windows x86_64, macOS arm64 and x86_64; Linux and macOS with OpenSSL bundled) and creates the Release with generated notes, the archives and `SHA256SUMS`. Tags with a hyphen (such as `v1.1.0-rc.1`) become pre-releases. Once you have checked the Release, publish to crates.io:

```bash
cargo publish
```

Leave crates.io for last: a version published there cannot be deleted, only yanked.

## Testing the workflow

The workflow was written without being able to run it on GitHub, so the first tag is the real test. Make the first one a candidate: put `version = "1.0.0-rc.1"` in `Cargo.toml`, push the tag `v1.0.0-rc.1` and check the four archives, `SHA256SUMS` and that the binary runs. Then delete the candidate's tag and Release.

## If something fails

Fix it, delete the tag and the Release, and push the tag again:

```bash
git push --delete origin v1.0.0
gh release delete v1.0.0 --cleanup-tag
```
