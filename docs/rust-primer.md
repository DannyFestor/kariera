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
| `make snapshots` | Shows each snapshot the last test run changed and asks whether to accept it |
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

## `Result`, `Option` and `?`

`Result<T, E>` is an enum with two cases, `Ok(T)` and `Err(E)`; `Option<T>` has `Some(T)` and `None`. `?` after a `Result` unwraps an `Ok` and returns from the function with the `Err`. Parsing a month in [`calendar.rs`](../crates/domain/src/calendar.rs) shows the usual chain:

```rust
fn from_str(text: &str) -> Result<Self, Self::Err> {
    let (year, month) = text.split_once('-').ok_or(ValueError::NotAMonth)?;
    let year = fixed_width_number(year, 4).ok_or(ValueError::NotAMonth)?;
    let month = fixed_width_number(month, 2).ok_or(ValueError::NotAMonth)?;
    Self::new(year, month).ok_or(ValueError::NotAMonth)
}
```

`split_once` returns an `Option`; `ok_or` turns its `None` into the error, and `?` returns that error. The last line has no `?`: its `Result` is the function's result. `map` changes the `Ok` value, `map_err` the error, and `and_then` chains a second step that can fail too.

`"2024-05".parse::<Month>()` calls this `from_str`, because `Month` implements the standard trait `FromStr`. Every value type in `domain` is parsed this way, so an invalid month cannot exist as a `Month`.

## Error types

An error is an enum with one case per way to fail, so the caller can `match` on it. [`thiserror`](https://docs.rs/thiserror) writes the `Display` text from an attribute on each case, as in [`value_error.rs`](../crates/domain/src/value_error.rs):

```rust
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValueError {
    #[error("must not be blank")]
    Blank,
    #[error("must be a whole number from {min} to {max}")]
    OutOfRange { min: u32, max: u32 },
    // …
}
```

`{min}` refers to the case's field. A field named `source` is the underlying error, as `io::Error` in `LoadError::Unreadable` of [`error.rs`](../crates/vault/src/error.rs). Only `cli` decides how an error is shown to the user; the other crates return it.

## Collecting every problem

`?` stops at the first error. Reading a note should report every bad field at once, so the note parsers in [`crates/vault/src/notes/`](../crates/vault/src/notes/) read each field into its own variable first and use `?` only when they build the value:

```rust
let name = fields.required("name");
let category = fields.required::<SkillCategory>("category");
// … every other field …
Ok(Skill {
    name: name?,
    category: category?,
    // …
})
```

Each `fields.required` records its problem in a collector shared by the note (`Problems` in [`problems.rs`](../crates/vault/src/problems.rs)) and returns `Err(Reported)`, a marker that says "already recorded". By the time the first `?` returns, every field has been read and every problem recorded. `fields.required("name")` has no type of its own: Rust infers it from the `Skill` field it ends up in.

Tests are the exception: a failing test should panic, so `clippy.toml` allows `unwrap`, `expect`, `panic!` and indexing inside `#[test]` functions and `#[cfg(test)]` modules, and only there. A helper function in a test file is neither, so it returns a `Result` and the test calls `expect` on it, as `kariera` does in [`crates/cli/tests/kariera.rs`](../crates/cli/tests/kariera.rs).

## `#[expect]` instead of `#[allow]`

An **attribute** such as `#[derive(Debug)]` or `#[test]` sits above an item and changes how it is compiled. `#[allow(lint)]` switches a lint off for an item and stays silent forever. It is denied here.

`#[expect(lint, reason = "…")]` switches the lint off too, but fails the build once the lint no longer fires, so a stale exception cannot linger, and it must say why. It is allowed only when the code is right and the lint is wrong for this one item, and the reason has to show that. Two kinds exist today. In [`crates/cli/src/reply.rs`](../crates/cli/src/reply.rs) the one crate allowed to print does so:

```rust
#[expect(
    clippy::print_stdout,
    reason = "the cli is the only crate that prints (ADR 0001)"
)]
fn write_stdout(text: &str) {
    println!("{text}");
}
```

And `Contract.contract_type` in [`contract.rs`](../crates/domain/src/contract.rs) starts with its struct's name, which `clippy::struct_field_names` reports; the field keeps the name because it is the schema's key.

## Modules and visibility

A crate is split into **modules**. `mod command;` in [`main.rs`](../crates/cli/src/main.rs) loads [`command.rs`](../crates/cli/src/command.rs) beside it. A module with submodules is a file `foo.rs` next to a directory `foo/`; `foo/mod.rs` is denied, so there is one layout.

Everything is private to its module unless marked: `pub(crate)` makes an item visible across its own crate, `pub` makes it part of the crate's API for other crates. `pub` is used only for what another crate actually uses (the `unreachable_pub` lint), so a crate's `pub` items are its whole interface.

`//!` at the top of a file documents the crate or module, and `///` documents the item below it. Neither is required: names carry the meaning, and a comment only appears where it says something the code cannot, such as a constraint or a reason (see `AGENTS.md`).

## Enums and `match`

An `enum` lists every case a value can be, and a case can carry data, like `Unknown(String)` in [`command.rs`](../crates/cli/src/command.rs). `match` has to handle every case, and a `_` arm that silently catches the rest is denied for enums (`wildcard_enum_match_arm`), so adding a case makes the compiler list every `match` that has to handle it.

## Traits and generics

A **trait** is a set of methods a type promises, like an interface. A **generic** function or type works for any type `T` that implements the traits it names, written `T: Trait`. In [`value.rs`](../crates/vault/src/value.rs) the trait `FromNode` says how a type is read from a YAML node, and one `impl` covers many types:

```rust
impl<T: FromScalar> FromNode for T { … }
impl<T: FromNode + PartialEq> FromNode for Vec<T> { … }
```

The first line reads every type that is written as one YAML value (a `Month`, a `Text`, …); the second reads a list of any readable type, so `Vec<Month>` and `Vec<Vec<Text>>` work without more code.

`Link<T>` in [`link.rs`](../crates/domain/src/link.rs) is a generic struct: a `Link<Skill>` and a `Link<Project>` hold the same note name, but the compiler keeps them apart, so a Project's `skills` cannot hold a link to an Organization. The `PhantomData` field exists only to use `T`, which the struct does not store.

## Trait objects

Generics are resolved at compile time: `fn show<T: Display>(value: T)` is compiled once for every `T` it is called with. A **trait object** decides at run time instead. `&dyn Translatable` in [`translatable.rs`](../crates/domain/src/translatable.rs) is a reference to any value whose type implements `Translatable`, without saying which, so one function can return "the Skill, the Project or the Organization of this name" and its caller needs only the trait's methods:

```rust
pub fn translatable(&self, note_type: NoteType, name: &NoteName) -> Option<&dyn Translatable> {
    match note_type {
        NoteType::Organization => self.organizations.get(name).map(as_translatable),
        NoteType::Skill => self.skills.get(name).map(as_translatable),
        // …
    }
}
```

`as_translatable` exists because `map` needs a function to name the conversion from `&Organization` to `&dyn Translatable`, and a bare `as` cast there is denied (`trivial_casts`).

## serde

[serde](https://serde.rs) turns data formats into Rust values. `#[derive(Deserialize)]` writes the code that fills a struct from any format serde supports; the `toml` crate supplies the format. The config file in [`config.rs`](../crates/vault/src/config.rs) is read this way:

```rust
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    vault: String,
}

let file: ConfigFile = toml::from_str(text)?;
```

A missing `vault` key or an unknown key is an error, not a silent default. The notes are not read with serde: their checks need the line of every value, whether it was quoted, and every problem at once instead of the first, so [`yaml.rs`](../crates/vault/src/yaml.rs) builds its own small tree and `FromNode` reads it.

## Macros

A name ending in `!` is a **macro**: code that writes code at compile time. `println!` and `format!` are macros of the standard library. [`fixed_list.rs`](../crates/domain/src/fixed_list.rs) defines its own with `macro_rules!`, so each fixed list of the schema is written once, as the table it is:

```rust
labelled_list!(pub RateUnit {
    Hour => "hour" ["Hourly rate", "時給", "Stundensatz"],
    Day => "day" ["Daily rate", "日給", "Tagessatz"],
    Month => "month" ["Monthly rate", "月額", "Monatssatz"],
});
```

The macro turns this into an enum, its slugs and its labels.

## Tests

- **Unit tests** sit at the bottom of the file they test, in a `#[cfg(test)] mod tests` module, which is compiled only for tests and can reach the file's private items. See [`command.rs`](../crates/cli/src/command.rs) and [`reply.rs`](../crates/cli/src/reply.rs).
- **Integration tests** live in `crates/<crate>/tests/` and use the crate from outside, like a user. [`crates/cli/tests/kariera.rs`](../crates/cli/tests/kariera.rs) runs the built binary.
- `make test` runs both with [cargo-nextest](https://nexte.st), then the examples in doc comments ("doc tests") with `cargo test --doc`, because nextest skips those.
- Tests run on the fictional vault in [`fixtures/vault/`](../fixtures/vault/), never on real data.
- A test that breaks a rule copies the fixture vault into a temporary folder, changes one note and compares the findings, as in [`test_vault.rs`](../crates/validate/src/test_vault.rs). The copy is deleted when the test value is dropped: `Drop` is the trait that runs code when a value goes out of scope.
- `make mutants` runs [cargo-mutants](https://mutants.rs): it changes the code (for example, replaces a function body with a default value) and reports every change the tests still pass with. It is slow and not part of `make check`.
- Two integration test files of one crate share helpers by including the same file with `#[path = "support/vault_copy.rs"] mod vault_copy;`, as [`crates/vault/tests/`](../crates/vault/tests/) does for the copy of the fixture vault. A helper file directly in `tests/` would be built as a test of its own, and `support/mod.rs` is denied like every `mod.rs`. Tests that write copy the fixture vault into cargo's scratch directory (`CARGO_TARGET_TMPDIR`) first.
- **Snapshot tests** with [insta](https://insta.rs) compare a whole value with a file written earlier, the snapshot. [`crates/vault/tests/fixture_vault.rs`](../crates/vault/tests/fixture_vault.rs) snapshots the whole fixture vault as read, with `insta::assert_debug_snapshot!`; the snapshots sit in `tests/snapshots/` beside the test and are committed. When a value changes, the test fails and insta writes the new value next to the snapshot as `.snap.new`. `make snapshots` shows each change and asks whether to accept it; `make snapshots-accept` accepts them all, so read every `.snap.new` first.

## Rules over the whole vault

`vault` reads each note on its own; [`kariera-validate`](../crates/validate/src/lib.rs) checks what only the whole vault can tell. [`checks.rs`](../crates/validate/src/checks.rs) lists one function per concern of the schema, all of the same type:

```rust
type Check = fn(&Context<'_>) -> Vec<Finding>;

const CHECKS: [Check; 15] = [placement::check, file_names::check, /* … */ profile::check];
```

A `fn(…) -> …` type is a plain function stored in an array, like a function pointer in C. `run` calls each one on the same [`Context`](../crates/validate/src/context.rs), which wraps the loaded vault with the lookups the checks share, and concatenates what they return with `flat_map`. A check never prints and never stops at the first problem: it returns every [`Finding`](../crates/validate/src/finding.rs), and `cli` decides how to show them.

`Context::notes::<Engagement>()` shows two things about generics. The turbofish `::<Engagement>` names the type that Rust cannot infer from the arguments, and the bound `T: Note + 'a` says the notes may be borrowed for as long as the vault is (`'a` is a lifetime: how long a reference stays valid). The result is a [`Found`](../crates/validate/src/context.rs), a note with its name and file, so a finding knows where to point.

Each check starts with the data it needs and ends in a chain of `filter_map`, which keeps the `Some` values of an `Option`-returning closure, so "this note is fine" is `None` and "this note breaks the rule" is `Some(Finding)`:

```rust
context.notes::<Project>().filter_map(|project| project_name(context, &project))
```

`validate` calls `Vault::inspect`, which keeps going past notes that do not read: the notes that read are the vault, the others come back as errors and become `Finding`s of rule `schema`. The checks run on what read; the two that need every note ask `context.is_complete()` first.

[`layout.rs`](../crates/validate/src/layout.rs) writes the frontmatter of each note type as `const` tables (`const PROJECT: &[Field] = &[…]`): the fields in the order of the schema, each with a `Shape` that says how its value is written. `const` data is built at compile time, and `&'static` references into it cost nothing at run time. The order check and the style check both walk these tables.

`kariera validate --json` is written by hand in [`json.rs`](../crates/cli/src/json.rs): the output has a fixed shape of strings, numbers, booleans, lists and objects, so the standard library is enough and no crate is added for it.

## When a check fails

| Failing check | Usually means |
|---|---|
| `fmt-check` | Run `make fmt`. |
| `lint` | A Clippy error; read it as above. |
| `deny` | A dependency outside the allowlist or outside the crate boundaries, a licence outside the allowlist, or a security advisory. |
| `machete` | A crate lists a dependency it never uses; remove it from its `Cargo.toml`. |
| `lint-shell` | shellcheck found a problem in a script under `scripts/`. |
| `test` | A test failed; nextest prints its output. A failed snapshot leaves a `.snap.new` to review with `make snapshots`. |
