# PROTOTYPE: Skill Sheet layout (throwaway)

The question: what should the Skill Sheet (スキルシート) look like, as XLSX and as PDF? Three variants, rendered from the same document model:

| Variant | Shape | PDF font |
|---|---|---|
| A | Agency format (Findy style): A4 portrait, profile, years per Skill in three columns, one tall row per Project with ● phase columns | BIZ UDPGothic |
| B | Overview list: A4 landscape, years per Skill as bars, one compact row per Project, technology split into category columns; the XLSX body has no merged cells | Noto Sans JP |
| C | Project blocks (geechs style): A4 portrait, profile beside a Skill table, then one labelled block per Project that never splits across pages | IBM Plex Sans JP |

**Verdict: C**, with the defaults listed on issue #9. A and B stay here as the record of what was compared.

## Files

- `crates/documents/prototype/skill_sheet_model.py`: reads a vault's notes into the model JSON, with the years per Skill computed from Project periods (overlaps counted once). Stands in for the `documents` crate.
- `crates/render-typst/prototype/skill-sheet-{a,b,c}.typ`: the PDF templates. They read the model from `sys.inputs.data`.
- `crates/render-xlsx/prototype/skill_sheet_xlsx.py`: the XLSX variants, written with Python's XlsxWriter, using only what rust_xlsxwriter also supports.

## Running it

Needs `uv`, the Typst CLI at the version the renderer embeds (0.15.1), and the font files in a folder (they are not in the repository; all three are SIL OFL 1.1).

```sh
uv run --with pyyaml python -I crates/documents/prototype/skill_sheet_model.py fixtures/vault 2026-11 tmp/model.json
typst compile --ignore-system-fonts --font-path <fonts> --root . --input data=/tmp/model.json \
  crates/render-typst/prototype/skill-sheet-c.typ tmp/skill-sheet-c.pdf
uv run --with xlsxwriter python -I crates/render-xlsx/prototype/skill_sheet_xlsx.py tmp/model.json c tmp/skill-sheet-c.xlsx
```
