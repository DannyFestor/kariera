# PROTOTYPE: 履歴書 layout (throwaway)

Not production code. It answers "what should the 履歴書 look like, and where does each field come from?" for issue #22. It lives on the branch `prototype/rirekisho` only.

`fixture.json` is fictional data (the owner of `fixtures/vault/`) in the shape proposed for a `Rirekisho` model in `documents`: every text already in Japanese and formatted, so the templates only lay it out.

| Variant | Idea |
|---|---|
| `variant-a.typ` | 厚生労働省様式: the ruled two-page form of the 2021 厚生労働省履歴書様式例 |
| `variant-b.typ` | 転職者用: ruled form, separate 学歴 / 職歴 / 免許・資格 / 語学 tables, separate 自己PR, 志望動機 and 本人希望 boxes |
| `variant-c.typ` | フリーフォーム: no ruled form, the look of the 職務経歴書, one 職歴 line per Engagement |

## Run

With the Typst CLI at the version the renderer embeds (0.15.1, see `../shokumu-keirekisho/README.md` on its branch for installing it into `tmp/`):

```sh
tmp/typst-cli/bin/typst compile --root . --ignore-system-fonts --font-path crates/render-typst/fonts \
  --input data=/crates/render-typst/prototype/rirekisho/fixture.json \
  crates/render-typst/prototype/rirekisho/variant-a.typ tmp/rirekisho/variant-a.pdf
```

Add `--input photo=/fixtures/personal/photo.png` for a photo.

## Outcome

The owner chose **variant A**. The decisions are on issue #22:

- **Personal block:** a 性別 value, plus two rows for 国籍 and 在留資格 under 生年月日.
- **Contact:** 連絡先 says 「同上」.
- **History and qualifications:** the 学歴, 職歴 and 免許・資格 wording.
- **Free sections:** the 自己PR and 語学 box, and the 本人希望 box built from the Profile's availability.
- **Missing values:** they leave empty cells.
- **Long histories:** the page-2 boxes share the height the history leaves.
