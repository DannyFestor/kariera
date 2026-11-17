# Crate boundaries

kariera is a Cargo workspace of eight crates under `crates/`, each with one job. A crate may depend only on the workspace crates its row allows:

| Crate | Job | May depend on |
|---|---|---|
| `domain` | The vault's glossary as types (Engagement, Contract, Project, Monthly Log, Skill, …). No I/O. | nothing in the workspace |
| `vault` | Reads the vault's notes (Markdown with YAML frontmatter) into domain types, and is the only crate that writes to the vault. | `domain` |
| `validate` | Checks the notes against the schema and against each other. | `domain`, `vault` |
| `documents` | Assembles each document's content (CV, 職務経歴書, 履歴書, Skill Sheet) per language, independent of the output format. | `domain` |
| `render-typst` | Renders documents to PDF with Typst embedded as a library. | `documents`, `domain` |
| `render-docx` | Renders documents to DOCX. | `documents`, `domain` |
| `render-xlsx` | Renders documents to XLSX. | `documents`, `domain` |
| `cli` | The `kariera` binary: parses arguments, wires the other crates together, prints, and sets the exit code. | every crate |

`vault` also writes, because it is the crate that knows what a note on disk looks like: the writes are the few the schema leaves to kariera (the `translated_from` value, ADR 0002; new notes from the vault's templates). They are small edits of one note's text through one helper, never a rewrite of a note from its domain value. No other crate touches the vault's files, so a change to how notes are written has one place to go.

Only `cli` prints. Every other crate returns values and errors, so it can be tested without capturing output, and what the user sees is decided in one place.

The table keeps each change in one place: `domain` without I/O is tested with plain values; the renderers never see the vault, so a new output format leaves reading and validation untouched; `documents` never sees a format, so the three renderers share one content model; `validate` reads through `vault` because it checks notes as they are on disk (file names, links, sources), which are not domain types.

## Enforcement

- Cargo lets a crate use only the crates its `Cargo.toml` lists.
- `crate-boundaries.toml` lists, for each workspace crate, the crates allowed to depend on it directly (cargo-deny `wrappers`). `make deny`, part of `make check` and the pre-push hook, fails on any other dependency.
- The lints `clippy::print_stdout` and `clippy::print_stderr`, and `disallowed-methods` on `std::io::stdout` and `std::io::stderr` in `clippy.toml`, deny printing everywhere. The cli prints through two functions in `crates/cli/src/reply.rs`, each with an `#[expect]` that names this ADR.

## Considered Options

- **One crate with modules**: module privacy hides items, but any module can still import any other, so the boundaries would rest on review alone.
- **A script over `cargo metadata`**: precise, but one more tool to write and test, where cargo-deny is already pinned and states the rules declaratively.
- **The boundaries inside `deny.toml`**: cargo-deny refuses a crate named in both its allowlist and its deny list, and the allowlist has to name the workspace crates to deny every crate it does not list. The boundaries therefore live in their own file, checked by a second cargo-deny run.

## Consequences

- A new workspace crate needs a row here, an entry in `crate-boundaries.toml`, and an entry in the allowlist in `deny.toml`.
- A third-party crate that belongs to one crate (Typst to `render-typst`, `docx-rs` to `render-docx`, `rust_xlsxwriter` to `render-xlsx`) gets a `wrappers` entry in `crate-boundaries.toml` when it is added, so no other crate can pick it up.
- cargo-deny warns about wrappers that do not depend on their crate yet; `make deny` switches that warning off, because those wrappers are permissions, not mistakes.
