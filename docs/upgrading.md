# Upgrading

Where each pinned item is pinned, how to bump it and what to run afterwards. Bump one item per commit.

## Rust toolchain

- **Pinned in**: `channel` in `rust-toolchain.toml`, and `rust-version` in the root `Cargo.toml`.
- **Bump**: set both to the new stable release (`rustup check` shows it). rustup installs it on the next cargo command.
- **Afterwards**: `make check`. A new Clippy brings new lints in the denied groups; fix what they report rather than switching them off.

## Tools from GitHub releases

cargo-deny, cargo-machete, cargo-nextest, cargo-insta, typstyle, gitleaks, lefthook, cog, jq and shellcheck.

- **Pinned in**: `tools/<tool>.version` (the release version) and `tools/<tool>.sha256` (one checksum per supported platform). The Makefile builds each download URL from them, and `scripts/install-pinned-binary.sh` refuses a download whose checksum differs.
- **Bump**: write the new version to `tools/<tool>.version`, then `make pin-<tool>`, which rewrites `tools/<tool>.sha256` from the digests GitHub records for the release (needs `gh`). `make pin-tools` does this for every tool.
- **Afterwards**: the next target that uses the tool downloads it. Check that `tools/<tool>.sha256` still lists four platforms (cargo-deny publishes no Intel Mac build, so it has three), then run the target below.

| Tool | Run afterwards |
|---|---|
| cargo-deny | `make deny`; read the changelog for new checks or config keys |
| cargo-machete | `make machete` |
| cargo-nextest | `make test` |
| cargo-insta | `make test`, then change a snapshot on purpose and run `make snapshots` |
| typstyle | `make fmt-typst` and review the diff: a new version may format templates differently |
| gitleaks | stage a change, then `make lint-secrets`; read the changelog for new default rules |
| lefthook | `make hooks-install`, then commit on a branch to see the hooks run |
| cog | commit on a branch to see the commit-msg hook run |
| jq | pipe a hook event into `make agenthook-pre-tool-use` |
| shellcheck | `make lint-shell` |

## cargo-mutants

- **Pinned in**: `tools/cargo-mutants.version`. Upstream publishes no Apple Silicon binary, so `bin/cargo-mutants` is built from crates.io with `cargo install --locked`, which cargo checks against the registry's checksums.
- **Bump**: write the new version to `tools/cargo-mutants.version`.
- **Afterwards**: `make mutants`, which rebuilds it.

## Dependencies

- **Pinned in**: `Cargo.lock`.
- **Bump**: `cargo update -p <crate>` for one crate, or `cargo update` for all.
- **Afterwards**: `make check`. cargo-deny fails on a new duplicate version or a new licence.
