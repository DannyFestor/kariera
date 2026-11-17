# Rust primer for kariera

A start for readers who know other languages but not Rust. It explains only what this repository uses today and grows with the code. Every example links to the file it comes from.

## The layout

```
Cargo.toml            the workspace: its members, shared package fields and the lint rules
Cargo.lock            exact versions of every dependency, written by cargo, committed
rust-toolchain.toml   the exact Rust version; rustup installs it on the first cargo command here
crates/<name>/        one crate each: its own Cargo.toml and a src/ directory
```

A **crate** is Rust's unit of compilation, like a Go module that builds one library or one binary. A **workspace** groups crates that share one `Cargo.lock` and one `target/` build directory. The library crates start at `src/lib.rs`; the binary crate `cli` starts at [`crates/cli/src/main.rs`](../crates/cli/src/main.rs) and is built as the `kariera` command (its `[[bin]]` entry in [`crates/cli/Cargo.toml`](../crates/cli/Cargo.toml)).

Which crate may use which is fixed in [ADR 0001](adr/0001-crate-boundaries.md) and checked by `make deny`. Rust enforces part of this by itself: a crate can only `use` the crates its `Cargo.toml` lists under `[dependencies]`.

## Everyday commands

| Command | Does |
|---|---|
| `make setup` | Once per clone: installs the pinned Rust version and the tools into `bin/`, and the git hooks |
| `make build` | Compiles the workspace in debug mode into `target/` |
| `cargo run -p kariera-cli -- --version` | Builds and runs `kariera`; everything after `--` goes to `kariera` |
| `make test` | Runs every test |
| `make fmt` | Formats the code |
| `make lint` | Runs Clippy, Rust's linter |
| `make check` | Everything that must pass before work is done |
| `make mutants` | Changes the code on purpose and reports each change no test noticed |
| `make install` | Installs `kariera` for this machine, into `~/.cargo/bin` |

## Reading a Clippy error

Clippy prints errors like this (from an `unwrap()` added on purpose):

```
error: used `unwrap()` on a `Result` value
  --> crates/domain/src/lib.rs:10:5
   |
10 |     text.parse().unwrap()
   |     ^^^^^^^^^^^^^^^^^^^^^
   |
   = note: if this value is an `Err`, it will panic
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.100.0/index.html#unwrap_used
   = note: requested on the command line with `-D clippy::unwrap-used`
```

- The first line says what is wrong. `-->` gives file, line and column.
- The `help` link names the lint (`unwrap_used`) and explains why it exists and what to write instead. Read it before changing anything.
- "requested on the command line with `-D …`" means the lint comes from this repository's rules: the `[workspace.lints]` table in [`Cargo.toml`](../Cargo.toml), which cargo hands to Clippy as `-D` (deny) flags. Thresholds such as the 40-line limit per function live in [`clippy.toml`](../clippy.toml).
- Fix the first error first: later ones are often caused by it.

## Errors instead of panics

A **panic** stops the program, like an uncaught exception. `unwrap()` and `expect()` panic when a `Result` holds an error or an `Option` holds nothing, and indexing (`list[3]`) panics when the index is out of range. All of these are denied outside tests. Instead, a function that can fail returns `Result<T, E>`, and its caller passes the error on with `?`, much like `if err != nil { return err }` in Go.

Tests are the exception: a failing test should panic, so `clippy.toml` allows `unwrap`, `expect`, `panic!` and indexing inside `#[test]` functions and `#[cfg(test)]` modules, and only there. A helper function in a test file is neither, so it returns a `Result` and the test calls `expect` on it, as `kariera` does in [`crates/cli/tests/kariera.rs`](../crates/cli/tests/kariera.rs).

## `#[expect]` instead of `#[allow]`

An **attribute** such as `#[derive(Debug)]` or `#[test]` sits above an item and changes how it is compiled. `#[allow(lint)]` switches a lint off for an item and stays silent forever. It is denied here.

`#[expect(lint, reason = "…")]` switches the lint off too, but fails the build once the lint no longer fires, so a stale exception cannot linger, and it must say why. It is allowed only when the code is right and the lint is wrong for this one item, and the reason has to show that. The only ones today are in [`crates/cli/src/reply.rs`](../crates/cli/src/reply.rs), where the one crate allowed to print does so:

```rust
#[expect(
    clippy::print_stdout,
    reason = "the cli is the only crate that prints (ADR 0001)"
)]
fn write_stdout(text: &str) {
    println!("{text}");
}
```

## Modules and visibility

A crate is split into **modules**. `mod command;` in [`main.rs`](../crates/cli/src/main.rs) loads [`command.rs`](../crates/cli/src/command.rs) beside it. A module with submodules is a file `foo.rs` next to a directory `foo/`; `foo/mod.rs` is denied, so there is one layout.

Everything is private to its module unless marked: `pub(crate)` makes an item visible across its own crate, `pub` makes it part of the crate's API for other crates. `pub` is used only for what another crate actually uses (the `unreachable_pub` lint), so a crate's `pub` items are its whole interface.

`//!` at the top of a file documents the crate or module, and `///` documents the item below it. Neither is required: names carry the meaning, and a comment only appears where it says something the code cannot, such as a constraint or a reason (see `AGENTS.md`).

## Enums and `match`

An `enum` lists every case a value can be, and a case can carry data, like `Unknown(String)` in [`command.rs`](../crates/cli/src/command.rs). `match` has to handle every case, and a `_` arm that silently catches the rest is denied for enums (`wildcard_enum_match_arm`), so adding a case makes the compiler list every `match` that has to handle it.

## Tests

- **Unit tests** sit at the bottom of the file they test, in a `#[cfg(test)] mod tests` module, which is compiled only for tests and can reach the file's private items. See [`command.rs`](../crates/cli/src/command.rs) and [`reply.rs`](../crates/cli/src/reply.rs).
- **Integration tests** live in `crates/<crate>/tests/` and use the crate from outside, like a user. [`crates/cli/tests/kariera.rs`](../crates/cli/tests/kariera.rs) runs the built binary.
- `make test` runs both with [cargo-nextest](https://nexte.st), then the examples in doc comments ("doc tests") with `cargo test --doc`, because nextest skips those.
- Tests run on the fictional vault in [`fixtures/vault/`](../fixtures/vault/), never on real data.
- `make mutants` runs [cargo-mutants](https://mutants.rs): it changes the code (for example, replaces a function body with a default value) and reports every change the tests still pass with. It is slow and not part of `make check`.
- **Snapshot tests** with [insta](https://insta.rs) are planned for generated output, such as a whole document. insta is not a dependency yet: whoever writes the first snapshot test adds it under `[dev-dependencies]` of that crate and to the allowlist in `deny.toml`, and pins `cargo-insta`, the tool that reviews changed snapshots.

## When a check fails

| Failing check | Usually means |
|---|---|
| `fmt-check` | Run `make fmt`. |
| `lint` | A Clippy error; read it as above. |
| `deny` | A dependency outside the allowlist or outside the crate boundaries, a licence outside the allowlist, or a security advisory. |
| `machete` | A crate lists a dependency it never uses; remove it from its `Cargo.toml`. |
| `lint-shell` | shellcheck found a problem in a script under `scripts/`. |
| `test` | A test failed; nextest prints its output. |
