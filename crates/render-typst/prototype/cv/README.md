# PROTOTYPE: CV layout (en, de)

Throwaway. It answered "what should the CV look like?" for issues #7 (PDF) and #8 (DOCX), and lives only on the branch `prototype/cv`. The renderer rewrites the chosen variant properly; nothing here is production code.

```sh
crates/render-typst/prototype/cv/render.sh        # the fixture vault, into tmp/cv/fixture/
crates/render-typst/prototype/cv/render.sh real   # the vault in ~/.config/kariera/config.toml, into tmp/cv/real/ (private)
```

The script fetches Typst 0.15.1 (the version the renderer will embed) and the fonts below into `tmp/` on the first run, exports the vault's CV content with the `prototype_cv_data` example of the `cli` crate, and writes a PDF and PNG pages per variant and language.

## Chosen: variant C, "Projects first"

`variant-c.typ`. A and B stay here as the alternatives that were rejected.

- **Page**: A4, margins 16 mm left and right, 14 mm top, 16 mm bottom. Footer: name · headline, page n / total.
- **Typography**: Inter for Latin text, Noto Sans JP for Japanese glyphs (Japanese Organization names print as they are). Body 8.8 pt; the name 28 pt bold; section headings 14 pt bold. Colours: header band `#232a3b`, accent `#c2410c`, chip background `#f1f1f4`.
- **Dates**: English `Oct 2023`, `since Oct 2023`, `Apr 2021 – Sep 2023`, `1 Apr 2026`; German `10/2023`, `seit 10/2023`, `04/2021 – 09/2023`, `01.04.2026`. Years of a Skill with one decimal (`4.2 y`, `4,2 J.`).

| Section | Prints |
|---|---|
| Header band (full bleed) | Profile `name`, `headline`; contact: location, phone, email (personal.yaml), `portfolio_url`, `github_url`; the photo (personal.yaml) on the German CV only |
| At a glance strip | `available_from` with `days_per_week_min`–`max`; human-language Skills with `level`; birth date and nationality (personal.yaml) on the German CV only |
| Profile | `## Summary` beside `## Strengths` |
| Selected projects | The 5 most recent Projects of all Engagements, ongoing first, then by `end`: `title` (`public_title` if `confidential`), period, `role` (`focus`), the Engagement's Client, `end_client` (never for a confidential Project), `industry`, `team_size`, the first paragraph of `## Description`, up to 3 `## Achievements`, `skills` as chips |
| Career | One row per Engagement, newest first: period, `role` (else the Engagement type), Client or employer, Engagement type, number of Projects when more than one |
| Skills | Skills used in a Project in the last 3 years, grouped by category, most years first, with computed years (schema §4). Provisional: the rule is being reworked (a Skill used outside any Project, such as private use, counts as not used) |
| Education, Certificates and licences | Two columns. Degree Qualifications: `labels`/`name`, `issuer`, `date`. Others: `labels`/`name`, `level`, `score`, `issuer`, `date` |

An untranslated German value prints the English one. On the real vault the CV runs to 3 pages in English and 4 in German; accepted for now.

## Fonts

All under the SIL Open Font License 1.1, which allows embedding them in the binary and in the PDF.

| Family | Version | Source | Used by |
|---|---|---|---|
| Inter | 4.1 | `rsms/inter` release `Inter-4.1.zip`, `extras/otf/` | C |
| Noto Sans JP | Subset OTF (JP) | `notofonts/noto-cjk`, `Sans/SubsetOTF/JP/` | C |
| IBM Plex Sans | 1.1.0 | `IBM/plex` release `@ibm/plex-sans@1.1.0` | B |
| IBM Plex Sans JP | Google Fonts | `google/fonts`, `ofl/ibmplexsansjp/` | B |
| IBM Plex Serif | 2.0.0 | `IBM/plex` release `@ibm/plex-serif@2.0.0` | A |
| Noto Serif JP | Subset OTF (JP) | `notofonts/noto-cjk`, `Serif/SubsetOTF/JP/` | A |

Noto Sans JP is about 4.5 MB per weight; Typst embeds only the glyphs a PDF uses.

## What the domain lacks

- Contact and personal details (email, phone, location, birth date, nationality, photo): `personal.yaml` has no schema yet. `personal-fixture.yaml` is the fictional shape the prototype reads.
- A Skill used outside any Project (private use) has no record, so its years and last use come out too low.
- No way to mark which Projects a CV features; the prototype takes the 5 most recent.
- No shorter CV text: the Profile and Engagement summaries are what pushes the real CV past two pages.
