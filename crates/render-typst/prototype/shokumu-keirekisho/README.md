# PROTOTYPE: 職務経歴書 and 履歴書 layouts (throwaway)

Not production code. It answers one question, "what should the 職務経歴書 (PDF + DOCX) and the 履歴書 look like?", for the renderer issues #7 (PDF) and #8 (DOCX). It lives on the branch `prototype/shokumu-keirekisho` only.

Three structurally different 職務経歴書 variants, all A4 portrait, all fed from the same JSON:

| Variant | Idea | Font |
|---|---|---|
| `variant-a.typ` | 定番の表組み: the agency form. One bordered table per Engagement (期間, 業務内容, 開発環境, 役割・体制) | BIZ UDPMincho, headings BIZ UDPGothic |
| `variant-b.typ` | サマリー先行・カード型: page 1 is an overview (要約, 得意分野, 資格, Skills with experience bars), then one card per Project | Noto Sans JP, now BIZ UDPGothic (`--input font=…`) |
| `variant-c.typ` | 一覧表・コンパクト: one continuous table, one row per Project, the six phases as ● columns | BIZ UDPGothic |

`common.typ` holds only data access and the date conventions (西暦 or 和暦, `--input era=wareki`).

## Outcome

The owner chose **variant B**, adjusted to fit two to four pages: full cards only for Projects that ended within the last two years or are ongoing (`--input detail-years=2`), older Projects one line each, no Engagement summary, denser Skills. BIZ UDPGothic everywhere, so the PDF matches the DOCX. The decisions and open questions are on issues #7 and #8.

`crates/render-docx/prototype/shokumu-keirekisho/` builds variant B as DOCX with docx-rs 0.4.22. It is a standalone crate outside the workspace, so the workspace's dependency policy stays untouched:

```sh
cd crates/render-docx/prototype/shokumu-keirekisho
cargo run -- ../../../../tmp/prototype/fixture/data.json "$(date +%F)" ../../../../tmp/prototype/fixture/variant-b.docx
```

## Run

```sh
cargo install --locked --version 0.15.1 --root tmp/typst-cli typst-cli   # once: the Typst version the renderer embeds
# once: the fonts into tmp/fonts (see below)
crates/render-typst/prototype/shokumu-keirekisho/render.sh               # the fixture vault → tmp/prototype/fixture/
ERAS="seireki wareki" crates/render-typst/prototype/shokumu-keirekisho/render.sh ~/path/to/vault real  # private preview → tmp/prototype/real/
```

The data comes from `crates/cli/examples/prototype_shokumu_keirekisho_data.rs`, which loads a vault through the `vault` crate and dumps the notes as JSON. The real content model belongs in `documents`.

## Fonts

All SIL Open Font License 1.1, so they may be embedded in the binary and in the PDF.

| Family | Files | Source |
|---|---|---|
| BIZ UDPGothic | `BIZUDPGothic-Regular.ttf`, `-Bold.ttf` | `github.com/google/fonts/tree/main/ofl/bizudpgothic` |
| BIZ UDPMincho | `BIZUDPMincho-Regular.ttf`, `-Bold.ttf` | `github.com/google/fonts/tree/main/ofl/bizudpmincho` |
| Noto Sans JP | `NotoSansJP-Regular.otf`, `-Bold.otf` | `github.com/notofonts/noto-cjk`, `Sans/SubsetOTF/JP` |

BIZ UDPGothic and BIZ UDPMincho ship with Windows 10 (1809) and later, so a DOCX that names them also shows the same fonts in Word on Windows.
