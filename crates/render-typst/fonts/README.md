# Embedded fonts

Compiled into the binary and embedded, as subsets, into every PDF. Both families are under the SIL Open Font License 1.1 (the `*-OFL.txt` files beside them), which allows bundling them with software and embedding them in documents.

| Files | Family | Source | Used for |
|---|---|---|---|
| `Inter-Regular.otf`, `Inter-SemiBold.otf`, `Inter-Bold.otf` | Inter 4.1 | `rsms/inter` release `Inter-4.1.zip`, `extras/otf/` | Latin text of the CV |
| `BIZUDPGothic-Regular.ttf`, `BIZUDPGothic-Bold.ttf` | BIZ UDPGothic | `google/fonts`, `ofl/bizudpgothic/` | Every Japanese glyph: the 職務経歴書, the Skill Sheet, and Japanese names in the CV |

BIZ UDPGothic is the one Japanese family of every document (owner's decision on issue #7). It ships with Windows 10 (1809) and later, so a DOCX or XLSX that names it shows the same font.
