# kariera

Builds career documents from an Obsidian vault of career notes: CVs, 職務経歴書, 履歴書 and Skill Sheets, in English, Japanese and German, as PDF, DOCX and XLSX.

The vault is the single source of truth. Every note is Markdown with YAML frontmatter, and kariera never overwrites anything you would edit by hand: every output can be deleted and rebuilt. What it adds to the vault is a new, mostly empty note from `kariera new`, which you then fill in.

## Status

Not usable yet. The project is being set up.

## Planned commands

| Command | Does |
|---|---|
| `kariera validate` | Checks every note against the schema and against the other notes (see below) |
| `kariera new <log\|project\|engagement>` | Creates a note from the vault's templates, with what the arguments say filled in: `new log --engagement <name> --month <YYYY-MM>`, `new project --engagement <name> --title <title>`, `new engagement (--client <name> \| --employer <name>) --type <type> --start <YYYY-MM>`. Refuses to overwrite a note. |
| `kariera build` | Builds every document in every language and format; `--doc` and `--lang` narrow it down |
| `kariera translations --stale` | Lists the notes and languages whose translation is empty or older than the note's English text |
| `kariera translations --mark <note> --lang <ja\|de>` | Records that the translation matches the English text as it is now |

## Configuration

kariera reads `~/.config/kariera/config.toml`:

```toml
vault = "~/Notes/Career"
```

| Key | Required | Meaning |
|---|---|---|
| `vault` | yes | The vault root: the folder that holds `Profile.md` and the note folders. An absolute path, or one starting with `~/` for the home directory. |

Any other key is an error, so a misspelt setting is not silently ignored. Personal data for the 履歴書 does not go here: it lives in `~/.config/kariera/personal.yaml`.

kariera reads the notes in `Organizations/`, `Engagements/`, `Contracts/`, `Projects/`, `Monthly Logs/`, `Skills/` and `Qualifications/` (with their subfolders) and `Profile.md` in the vault root. `kariera new` also reads `Templates/` (`log` uses `Templates/monthly-log.md`, `project` `project.md`, `engagement` `engagement.md`). `kariera validate` also reads the frontmatter of the other Markdown files, to find notes in the wrong place, and checks that the files Sources cite exist. It writes only two things: a new note from `kariera new`, and the `translated_from` value of one note, set by `kariera translations --mark`; the fingerprint it stores is defined in [ADR 0002](docs/adr/0002-translation-fingerprints.md).

## `kariera validate`

```sh
kariera validate            # for people
kariera validate --json     # for programs and AI agents
```

Checks the vault named in the config against the note schema and against itself, and reports each problem with its file, line and field. Exit code 1 when there is an error, 0 when there are none or only warnings. A command line it does not understand exits 2, and a vault that cannot be read at all (no config, an unreadable folder) is reported on stderr and exits 1.

```text
Skills/Go.md:4: error [yaml-style] field `aliases` must be written in block style, not as `[…]` or `{…}`
Monthly Logs/2025/2025-07 Globex Log.md:7: warning [hours] field `project_hours` adds up to 150.25 h, but `hours` is 152.5: …
1 error, 1 warning in 43 notes.
```

`--json` prints one object: `errors`, `warnings`, `notes_checked`, `complete` and `findings`, each finding with `path`, `line` (or `null`), `severity` (`error` or `warning`), `rule`, `location` (`kind` and `name`) and `message`. The `rule` names what the finding is about: `schema`, `folder`, `file-name`, `duplicate-name`, `field-order`, `yaml-style`, `period`, `link-role`, `contract`, `rate`, `hours`, `alias`, `body`, `source`, `confidentiality`, `forbidden-content`, `profile`.

Every note is first read on its own (fields, values, links, body sections; rule `schema`). The other rules run over the notes that read: file names and folders (also Markdown elsewhere in the vault whose frontmatter has a `type`), field order and YAML style, periods, link roles, contracts and rates, `hours` against `project_hours`, Sources (the cited file must exist, compared in Unicode NFC), body content, the confidentiality of Projects, pull request URLs, and the Profile. When some notes could not be read, `complete` is `false`: the rules that need every note (a Contract listed by exactly one Engagement, exactly one Profile) did not run.

Only the `hours` rule has warnings: `project_hours` that do not add up to `hours` without a Source `note` starting with `Hours mismatch:`, and a Monthly Log that cites no invoice, timesheet or work report.

`make check-vault` runs it on the vault named in `~/.config/kariera/config.toml` (or `KARIERA_CONFIG`).

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
