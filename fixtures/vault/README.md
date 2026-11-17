# Fixture vault

A fictional copy of the career vault for the tests: a fictional person, Robin Beispiel, with a fictional trade name (Example Works) and fictional Organizations. It holds no real person, company or project, and every Source path points to a file that does not exist.

kariera reads only the Markdown files in the note folders and `Profile.md`, so this file, `Templates/`, the canvas `Projects/Acme - Shop.canvas` and the hidden `Projects/.draft.md` are part of the fixture too: the tests check that none of them is read as a note.

## What it covers

- Every note type, and every optional field of each type on at least one note.
- Every Organization kind, Skill category, Qualification category, Phase and Source kind.
- Engagements: an internship, an employment with a Contract, a finished freelance Engagement, and an ongoing one (`2023-10 Globex`) whose work was first placed by an Agency and then contracted directly.
- Contracts: one via an Agency (`2023-10 Globex via Fabrikam Contract`), open-ended, by the hour with a `rate_history`, by the month, and an employment Contract without a rate.
- A confidential Project with a `public_title` (`Acme - Ledger`), and a non-confidential one that names its End Client (`Acme - Shop`).
- Monthly Logs with `project_hours` split over two Projects, once adding up to `hours` and once not, with an `Hours mismatch:` Source note.
- Japanese and German text in fields and body sections, translated, untranslated and partly translated; `translated_from` fingerprints; `reviewed: true` and `false`.
- Names from schema §1.1: `C Sharp` with the alias `C#`, and `Müller Logistik` with an umlaut.
- A Skill alias equal to one of its `labels` (`Example Charts`, schema §2.6).

## Written on purpose

- `Engagements/2019-04 Müller Logistik.md` writes its `employer` link in Unicode NFD, as macOS does; the file name and the link target of every other note are NFC. kariera compares names in NFC (schema §1.1), so the link resolves. An editor that normalizes on save would undo this.
- The `translated_from` values are made up, so none of them equals a real fingerprint (ADR 0002): every translated note counts as stale until a test marks it in a copy of the vault.
