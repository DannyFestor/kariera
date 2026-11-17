# kariera

Builds career documents from an Obsidian vault of career notes: CVs, 職務経歴書, 履歴書 and Skill Sheets, in English, Japanese and German, as PDF, DOCX and XLSX.

The vault is the single source of truth. Every note is Markdown with YAML frontmatter, and kariera never writes anything you would edit by hand: every output can be deleted and rebuilt.

## Status

Not usable yet. The project is being set up.

## Planned commands

| Command | Does |
|---|---|
| `kariera validate` | Checks every note against the schema, finds broken links and missing sources |
| `kariera new <log\|project\|engagement>` | Creates a note from the vault's templates |
| `kariera build` | Builds every document in every language and format; `--doc` and `--lang` narrow it down |
| `kariera translations --stale` | Lists notes whose English text changed after their translations |

## Configuration

kariera reads `~/.config/kariera/config.toml`:

```toml
vault = "~/Notes/Career"
```

| Key | Required | Meaning |
|---|---|---|
| `vault` | yes | The vault root: the folder that holds `Profile.md` and the note folders. An absolute path, or one starting with `~/` for the home directory. |

Any other key is an error, so a misspelt setting is not silently ignored. Personal data for the 履歴書 does not go here: it lives in `~/.config/kariera/personal.yaml`.

kariera reads the notes in `Organizations/`, `Engagements/`, `Contracts/`, `Projects/`, `Monthly Logs/`, `Skills/` and `Qualifications/` (with their subfolders) and `Profile.md` in the vault root. It reads nothing else in the vault and never writes to it.

## Design

- Written in Rust, as a workspace of small crates whose dependency rules are enforced by Cargo.
- PDF rendering uses [Typst](https://typst.app) embedded as a library: one binary, no browser, no TeX installation.
- DOCX via `docx-rs`, XLSX via `rust_xlsxwriter`.
- The build is deterministic and offline: no LLM calls, no network.
- Tests run against a fictional fixture vault in this repository, never against real personal data.

## Development

Needs `make`, `git`, `curl` and [rustup](https://rustup.rs); everything else is pinned and installed into this checkout.

```sh
make setup   # the pinned Rust toolchain, the tools in bin/ and the git hooks
make check   # formatting, Clippy, cargo-deny, cargo-machete, shellcheck and the tests
make help    # every target
```

New to Rust? Start with [`docs/rust-primer.md`](docs/rust-primer.md). AI agents follow [`AGENTS.md`](AGENTS.md).

## Setup still to do

The toolchain, lint, hook and test setup is done. Remaining:

1. Run `/setup-matt-pocock-skills` in this repository.
