# Translation fingerprints

The schema (§1.6.1) marks a translation as out of date by comparing the `translated_from` value stored in the note with a fingerprint of the note's English translatable content. The vault has no git and Syncthing changes file times, so neither a diff nor a modification time can tell. The algorithm belongs to kariera.

**Changing any rule below changes the fingerprint of every note and marks every translation stale.** A change therefore needs a new ADR that supersedes this one, and the owner re-marks every translation afterwards.

## What is fingerprinted

The English text the translations are made from, as the schema's tables name it. A part that is empty or absent is left out; the order is that of the schema's table (frontmatter fields first, then body sections).

| Note type | Parts, in order |
|---|---|
| organization | `name`, `## Description` |
| engagement | `role` (`en`), `## Summary` |
| project | `title`, `public_title`, `role`, `focus`, `industry` (each `en`), `## Description`, `## Achievements` |
| skill | `name`, `level` (`en`) |
| qualification | `name` |
| profile | `headline` (`en`), `## Summary`, `## Strengths`, `## Self-PR` |

`name` counts where the type has `labels` (organization, skill, qualification), because `labels` is the translation of `name`. An organization's `labels` is its legal name in that language, so a renamed organization needs its labels reviewed. A profile's `name` is a proper noun and is not translated. Contracts and monthly logs have no `translated_from`. The Japanese and German values, `aliases`, `issuer` and every other field are not part of it: editing them never makes a translation stale. Bullets enter as `- ` plus the bullet, one per line; paragraphs as written.

## The algorithm

1. Normalize the text of each part: CR LF and lone CR become LF; Unicode NFC; whitespace at the end of each line removed; blank lines before the first and after the last line removed. Spaces inside a line and blank lines between lines stay.
2. Write each part as `<part>:<length>:<text>\n`, where `<part>` is the frontmatter key or the body heading (`role`, `## Description`) and `<length>` is the byte length of the normalized text in decimal. The length keeps text from imitating the start of the next part.
3. Join the parts, hash the UTF-8 bytes with SHA-256, and keep the first 6 bytes as 12 lowercase hex digits.

Anyone can recompute it without kariera: a skill named `Go` with no other English content is `printf 'name:2:Go\n' | shasum -a 256`, whose first 12 digits are `007a4f991fa0`.

SHA-256 is not needed for security. It is chosen because it is fixed, published, available in every language and in a shell, and 48 bits make an accidental collision between two edits of one note negligible. The `sha2` crate computes it; the algorithm is short, but a hand-written hash has no place in a project that treats its output as frozen.

## Marking and listing

- A translation is stale when its stored fingerprint is empty or differs from the current one. An empty value also covers a note that never had a `translated_from` key.
- `kariera translations --mark <note> --lang <ja|de>` writes the current fingerprint into `translated_from.<lang>`. It edits the note's text and parses nothing back out, so every other byte stays: the frontmatter is not read into a YAML tree and written out again, which would reformat the owner's file.
  - The value of an existing `<lang>:` line is replaced, and the whole line with it: a comment after the value is lost.
  - A missing `<lang>:` line is added inside the `translated_from:` block: `ja` before the first entry, `de` after the last. Only the marked language is written; the other is left as it is, absent or empty.
  - A missing block is added right before the first of `sources:` and `reviewed:`, which is where the field order of every note type with `translated_from` puts it (schema §2).
  - `translated_from:` with a value on its own line (`null`, `{}`) is refused: the schema asks for block style.
  - The new text replaces the file in one step (a temporary sibling, then a rename) and keeps its permissions, so a crash cannot leave half a note in a vault without git.
- `kariera translations --stale` lists note name and language, with `empty` or `differs`, for each stale translation. It is a report: the exit code does not depend on what it finds.

## Where it lives

`domain` states what is translatable (`Translatable`, `Fingerprint`, `TranslationLanguage`): it knows the schema's tables and does no hashing. `vault` normalizes, hashes and writes, because it owns NFC and the notes on disk. `cli` parses the options and prints. See ADR 0001 for why `vault` also writes.

## Considered Options

- **Hash the raw file sections instead of the parsed values.** Independent of the parser, but then each type's frontmatter fields still need the type's knowledge, and a reformatted YAML value would change the fingerprint.
- **Hash the whole English side as one text.** Swapping the `role` and `focus` texts would keep the fingerprint although each translation is now wrong; the part names prevent that.
- **Parse and re-serialize the frontmatter to write the value.** Simplest to code, but it rewrites quoting, comments and line breaks of the owner's notes.
- **A shorter or longer digest.** 12 digits are short enough to read in a note and long enough that two different states of one note will not meet.
