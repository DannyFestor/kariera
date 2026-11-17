# kariera

A Rust CLI that builds career documents from an Obsidian vault. See `README.md` for what it does and `docs/adr/` for the decisions behind it.

## Hard rules

1. **Finish only when `make check` passes.** The Stop hook runs it whenever the working tree has changes and holds the end of the turn until it passes.
2. **Never write `#[allow(…)]` or `#![allow(…)]`.** Fix the code. If a lint is wrong for one item, put `#[expect(lint, reason = "…")]` on that item alone, with a reason a reviewer can check. "To make it compile" or "Clippy is too strict" is not a reason. Never relax a lint or threshold in `Cargo.toml`, `clippy.toml` or `deny.toml`.
3. **No new dependency without a reason.** Prefer the standard library. A new crate needs an entry in the `[bans] allow` list of `deny.toml` with a `reason`, a licence already on the licence allowlist, and a sentence in your reply saying why it is needed. Dev-only crates (such as `insta`) go in `[dev-dependencies]`.
4. **Respect the crate boundaries** in `docs/adr/0001-crate-boundaries.md`. `domain` has no I/O. Only `cli` prints. `make deny` fails on a dependency the ADR forbids.
5. **No personal data and no real vault content in this repository.** Tests use the fictional vault in `fixtures/vault/`. Personal data lives in `~/.config/kariera/personal.yaml` on the owner's machines only. `make check-vault` is the only thing that touches the real vault, and only the owner runs it.
6. **Never skip or bypass the git hooks** (`--no-verify`, `-n`, `LEFTHOOK=0`, `core.hooksPath`). Never commit or push to `main`; work on a branch.
7. **Commit messages are Conventional Commits** (`feat(cli): …`, `fix(vault): …`), checked by cog.
8. **Run everything through `make`.** `make help` lists the targets. Do not call the tools in `bin/` directly.
9. **Comments say why, never what.** Names, types and tests carry the meaning. Write a comment only for something the code cannot say: a constraint, a reason, a non-obvious consequence, a link to an ADR. Never write a comment or doc comment that restates the item's name, signature or body (`/// Parses the arguments` above `fn parse`). When a comment seems necessary to explain what code does, rename or split the code instead.

## How the checks reach you

- Before an edit, a hook denies `#[allow]` and edits to `Cargo.lock`, and asks the human before an edit to a guardrail file (`Cargo.toml`, `deny.toml`, `crate-boundaries.toml`, `clippy.toml`, `rustfmt.toml`, `rust-toolchain.toml`, `Makefile`, `lefthook.yml`, `scripts/`, `tools/`, `docs/adr/`, this file).
- After each edit to a Rust file, a hook formats it and runs Clippy on the workspace with dead code tolerated. Fix what it reports before going on.
- At the end of the turn, `make check` runs: formatting, Clippy with every warning as an error, cargo-deny, cargo-machete, shellcheck, and the tests.

## Navigation

- `docs/rust-primer.md`: how the Rust in this repository works, for readers new to Rust. Keep it accurate when you change a pattern it describes.
- `docs/upgrading.md`: bumping the toolchain or a pinned tool.
- `docs/adr/`: one file per decision.
