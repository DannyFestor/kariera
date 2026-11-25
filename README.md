# kariera

Builds career documents from an Obsidian vault of career notes: CVs, 職務経歴書, 履歴書 and Skill Sheets, in English, Japanese and German, as PDF, DOCX and XLSX.

The vault is the single source of truth. Every note is Markdown with YAML frontmatter, and kariera never overwrites anything you would edit by hand: every output can be deleted and rebuilt. What it adds to the vault is a new, mostly empty note from `kariera new`, which you then fill in.

## Status

Usable. `kariera build` writes the CV (English, German), the 職務経歴書, the 履歴書 and the Skill Sheet, and public PDFs of the CV and the 職務経歴書 for a website.

## Install

```sh
make setup     # once: the pinned toolchain and tools
make install   # the kariera binary, into cargo's bin directory
```

Then write `~/.config/kariera/config.toml` (see [Configuration](#configuration)).

## Commands

| Command | Does |
|---|---|
| `kariera validate` | Checks every note against the schema and against the other notes (see below) |
| `kariera new <log\|project\|engagement>` | Creates a note from the vault's templates, with what the arguments say filled in: `new log --engagement <name> --month <YYYY-MM>`, `new project --engagement <name> --title <title>`, `new engagement (--client <name> \| --employer <name>) --type <type> --start <YYYY-MM>`. Refuses to overwrite a note. |
| `kariera build` | Builds every document in every language it is written in, as PDF, as DOCX where there is a layout (the CV and the 職務経歴書) and as XLSX (the Skill Sheet); the 履歴書 is a PDF only. Also builds public PDFs of the CV and the 職務経歴書. `--doc` and `--lang` narrow it down (see below) |
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

Any other key is an error, so a misspelt setting is not silently ignored. Personal data does not go here, and never into the vault: it lives in `~/.config/kariera/personal.yaml`.

### `personal.yaml`

The contact and personal details the documents print. The file and every key are optional; any other key is an error.

```yaml
email: robin@example.com
phone: +81 90 0000 0000
location: Sapporo, Japan
birth_date: 1990-01-01
nationality:
  en: German
  ja: ドイツ
  de: deutsch
photo: photo.jpg
name_kana: ロビン　ベイシュピール
gender: 男
postal_code: 001-0000
address: 北海道札幌市北区北0条西0丁目0-0
address_kana: ほっかいどう さっぽろし きたく きた0じょう にし0ちょうめ
residence_status: 技術・人文知識・国際業務
work_restriction: 在留資格に基づく就労活動のみ可
residence_expiry: 2028-03-31
residence_note: 在留期間更新申請中
public:
  email: hello@example.com
  location: Japan
```

| Key | Printed in |
|---|---|
| `email`, `phone` | The CV's header, English and German, and the 履歴書 (現住所) |
| `location` | The CV's header, English and German |
| `birth_date` (`YYYY-MM-DD`) | The German CV, as German readers expect it, and the 履歴書 with the age on the build date |
| `nationality` | The German CV and the 履歴書. A map of `en`, `ja` and `de`, with `en` required: each document prints its language's entry, else `en` |
| `photo` | The German CV and the 履歴書. A PNG, JPEG, GIF, SVG or WebP file, relative to the folder of `personal.yaml`, absolute, or starting with `~/` |
| `name_kana`, `gender` | The 履歴書: ふりがな of the name, in katakana, and 性別 |
| `postal_code`, `address`, `address_kana` | The 履歴書: 現住所 with 〒, and its ふりがな |
| `residence_status`, `work_restriction`, `residence_expiry` (`YYYY-MM-DD`), `residence_note` | The 履歴書, for a foreign applicant: 在留資格 with the work restriction in brackets, 在留期限, and a note such as an application in progress. The 国籍 and 在留資格 rows are left out when `nationality`, `residence_status`, `residence_expiry` and `residence_note` are all missing |
| `public` | The public documents (see [`kariera build`](#kariera-build)), and nothing else: a map of `email` and `location`, both optional, and no other key. The location is meant to be country-level, such as `Japan`. A public document prints these two and no other key of the file, so without `public` it prints no contact details at all |

Every key but `public` is private: it is printed only in the documents of `Generated/`, never in those of `Generated/public/`.

kariera reads the notes in `Organizations/`, `Engagements/`, `Contracts/`, `Projects/`, `Monthly Logs/`, `Skills/` and `Qualifications/` (with their subfolders) and `Profile.md` in the vault root. `kariera new` also reads `Templates/` (`log` uses `Templates/monthly-log.md`, `project` `project.md`, `engagement` `engagement.md`). `kariera validate` also reads the frontmatter of the other Markdown files, to find notes in the wrong place, and checks that the files Sources cite exist. It writes only three things: a new note from `kariera new`; the `translated_from` value of one note, set by `kariera translations --mark` (the fingerprint it stores is defined in [ADR 0002](docs/adr/0002-translation-fingerprints.md)); and the documents of `kariera build`, into `Generated/` and `Generated/public/`, which it replaces on every build.

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

## `kariera build`

```sh
kariera build                       # every document in every language it is written in
kariera build --doc cv              # the CV in English and German
kariera build --lang ja             # the 職務経歴書, the 履歴書 and the Skill Sheet
kariera build --doc rirekisho       # the 履歴書
kariera build --doc cv --lang de    # the German CV only, private and public
```

Writes each document as PDF into the vault's `Generated/`, replacing the file of the last build, and prints each file, a PDF with its page count. The CV and the 職務経歴書 also come as DOCX, and the Skill Sheet as XLSX, because agencies edit them; each is written beside the PDF with the same name and its own extension, and `--doc` and `--lang` choose all formats alike:

| Document | `--doc` | Languages | File, for a Profile `name` of Jane Example |
|---|---|---|---|
| CV | `cv` | `en`, `de` | `Generated/CV_Jane_Example.pdf`, `Generated/Lebenslauf_Jane_Example.pdf` |
| 職務経歴書 | `shokumu-keirekisho` | `ja` | `Generated/職務経歴書_Jane_Example.pdf` |
| 履歴書 | `rirekisho` | `ja` | `Generated/履歴書_Jane_Example.pdf` |
| Skill Sheet | `skill-sheet` | `ja` | `Generated/スキルシート_Jane_Example.pdf` and `Generated/スキルシート_Jane_Example.xlsx` |

A file is named by the document's title in its language and the Profile `name`, with spaces as `_`.

The DOCX files are made to be edited: headings, lists and tables are Word's own, and no text is an image. They name their fonts and do not embed them: Calibri for Latin text on the CV (it ships with Office), where the PDF uses Inter, and BIZ UDPGothic for Japanese (it ships with Windows 10 1809 and later; where it is missing, Word substitutes another font). The author is the Profile `name` and the document language is `en`, `de` or `ja`. The 職務経歴書 shows the time a Skill was used as text, where the PDF also draws a bar. Like the PDFs, the same notes, `personal.yaml` and date give the same bytes.

- **Public documents:** every build also writes the CV (English and German) and the 職務経歴書 into `Generated/public/`, for the owner's website, as PDF only and with the same file names as the private PDFs; `--doc` and `--lang` choose them like the others. A public document is the private one with other personal details: of `personal.yaml` it prints only the `email` and `location` of its `public` map, so it has no phone, address, birth date, nationality or photo. There is no public 履歴書 or Skill Sheet.
- **Skills:** a skill table shows a Skill that at least one Project links and whose last use falls within the document's window: 3 years for the CV, 5 for the 職務経歴書 and the Skill Sheet. The last use is the later of its last Project month and its `last_used`. Its years are the months of its Projects, with months that two Projects share counted once; `last_used` never adds to them. Human languages are listed apart, not in the skill tables.
- **Confidential Projects** print their `public_title`, and never an End Client.
- **The 履歴書** is the ruled two-page form of the 2021 厚生労働省履歴書様式例, with every year in 西暦. 学歴 lists the degree Qualifications (入学 at `start`, 卒業 at `date`) by the school their `issuer` links, 職歴 the Engagements with wording by `engagement_type` and the Agencies of a freelance Engagement, and 免許・資格 every other Qualification. An own-business Engagement of the Profile's `business` prints 「個人事業 … 開業」 and no 「廃業」, even when it has ended, because the business still runs; 職歴 always closes with 「現在に至る」, since the Profile always names a `business`. The 志望の動機 box prints the Profile's Self-PR and the human languages, and 本人希望記入欄 its `available_from` and days per week. A value missing from `personal.yaml` leaves its cell empty; the form keeps every label and line.
- **Organizations** print their `labels` for the document's language, else their `name`.
- **The Skill Sheet XLSX** is real cells, with no image: the same sections and order as the PDF, on a grid of twelve columns, set in BIZ UDPゴシック (an XLSX cannot embed fonts, so the font must be on the machine that opens it; it ships with Windows). It prints on A4 portrait with page numbers, and a page break before each Project block that would not fit keeps the block on one page. Excel does not fit a row to its text when it opens a generated file, so each row's height is estimated from its text and column width; if a row looks too tall or too short in your Excel, resize it by hand. The scale is the sheet's own (no "fit to one page wide"), because Excel ignores manual page breaks under that option.
- **Missing translations:** where a Japanese or German text is empty, the document prints the English one, and the build ends with a warning naming the note and the language. It still exits 0.

The same notes, `personal.yaml` and date give the same PDF, DOCX and XLSX bytes, the public PDFs included. The date printed (作成日, 更新日) is today's date in the machine's time zone.

## Design

- Written in Rust, as a workspace of small crates whose dependency rules are enforced by Cargo.
- PDF rendering uses [Typst](https://typst.app) embedded as a library: one binary, no browser, no TeX installation.
- The PDF fonts are embedded in the binary: Inter for Latin text and BIZ UDPGothic for Japanese, both under the SIL Open Font License 1.1 (`crates/render-typst/fonts/`).
- DOCX via `docx-rs` (it has no setter for the author or the document language, so `render-docx` replaces those parts of the file after docx-rs has written them; its `zip` has known flaws in extracting archives, which kariera never does: `deny.toml` lists them), XLSX via `rust_xlsxwriter`.
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
