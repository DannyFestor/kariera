//! Writes one `translated_from` entry into a note's text and leaves every other byte alone. The frontmatter is not
//! parsed and written again: that would reformat the owner's YAML.
//!
//! Which lines change is fixed in ADR 0002.

use kariera_domain::{Fingerprint, TranslationLanguage};
use thiserror::Error;

use crate::Problem;
use crate::raw_note::split;

const KEY: &str = "translated_from";
const INDENT: &str = "  ";

/// `translated_from` stands right before the first of these in the field order of every type that has it
/// (schema §2).
const KEYS_AFTER: [&str; 2] = ["sources", "reviewed"];

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EditError {
    #[error("has a frontmatter that {}", .0.message)]
    NoFrontmatter(Problem),
    #[error(
        "has `translated_from` with a value on its own line; write it as a block with `ja:` and `de:` below"
    )]
    NotABlock,
    #[error("has none of the fields `sources` and `reviewed` to put `translated_from` before")]
    NoFieldAfter,
}

pub(crate) fn set(
    text: &str,
    language: TranslationLanguage,
    fingerprint: &Fingerprint,
) -> Result<String, EditError> {
    let parts = split(text).map_err(EditError::NoFrontmatter)?;
    let mut lines: Vec<String> = parts
        .yaml
        .split_inclusive('\n')
        .map(str::to_owned)
        .collect();
    set_in_lines(&mut lines, language, fingerprint)?;
    Ok(parts.with_yaml(&lines.concat()))
}

fn set_in_lines(
    lines: &mut Vec<String>,
    language: TranslationLanguage,
    fingerprint: &Fingerprint,
) -> Result<(), EditError> {
    match lines.iter().position(|line| is_key(line, KEY)) {
        Some(header) => set_in_block(lines, header, language, fingerprint),
        None => add_block(lines, language, fingerprint),
    }
}

fn add_block(
    lines: &mut Vec<String>,
    language: TranslationLanguage,
    fingerprint: &Fingerprint,
) -> Result<(), EditError> {
    let next_field = lines
        .iter()
        .position(|line| KEYS_AFTER.iter().any(|key| is_key(line, key)))
        .ok_or(EditError::NoFieldAfter)?;
    let block = [format!("{KEY}:\n"), entry(INDENT, language, fingerprint)];
    lines.splice(next_field..next_field, block);
    Ok(())
}

fn set_in_block(
    lines: &mut Vec<String>,
    header: usize,
    language: TranslationLanguage,
    fingerprint: &Fingerprint,
) -> Result<(), EditError> {
    let is_bare = lines
        .get(header)
        .is_some_and(|line| line.trim_end() == format!("{KEY}:"));
    if !is_bare {
        return Err(EditError::NotABlock);
    }
    let first_entry = header + 1;
    let entry_count = lines
        .iter()
        .skip(first_entry)
        .take_while(|line| line.starts_with([' ', '\t']))
        .count();
    let entries = first_entry..first_entry + entry_count;
    let indent = lines
        .get(first_entry)
        .filter(|_| entry_count > 0)
        .map_or(INDENT, |line| indentation(line))
        .to_owned();
    let existing = entries
        .clone()
        .find(|&at| lines.get(at).is_some_and(|line| is_entry(line, language)));
    match existing {
        Some(at) => replace(lines, at, entry(&indent, language, fingerprint)),
        None if language == TranslationLanguage::Japanese => {
            lines.insert(first_entry, entry(&indent, language, fingerprint));
        }
        None => lines.insert(entries.end, entry(&indent, language, fingerprint)),
    }
    Ok(())
}

fn replace(lines: &mut [String], at: usize, line: String) {
    if let Some(old) = lines.get_mut(at) {
        *old = line;
    }
}

fn entry(indent: &str, language: TranslationLanguage, fingerprint: &Fingerprint) -> String {
    format!("{indent}{}: {fingerprint}\n", language.code())
}

fn is_key(line: &str, key: &str) -> bool {
    line.strip_prefix(key)
        .is_some_and(|rest| rest.starts_with(':'))
}

fn is_entry(line: &str, language: TranslationLanguage) -> bool {
    is_key(line.trim_start(), language.code())
}

fn indentation(line: &str) -> &str {
    line.get(..line.len() - line.trim_start().len())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use kariera_domain::{Fingerprint, TranslationLanguage};

    use super::{EditError, set};

    const JAPANESE: TranslationLanguage = TranslationLanguage::Japanese;
    const GERMAN: TranslationLanguage = TranslationLanguage::German;

    fn fingerprint() -> Fingerprint {
        "0123456789ab".parse().unwrap()
    }

    fn note(translated_from: &str) -> String {
        format!(
            "---\ntype: skill\nname: Go\nlabels:\n  ja: ゴー\n{translated_from}sources:\n  - kind: recalled\n    note: \
             \"owner: used it\"\nreviewed: false\n---\nA body.\n"
        )
    }

    fn marked(before: &str, language: TranslationLanguage) -> String {
        set(&note(before), language, &fingerprint()).unwrap()
    }

    #[test]
    fn a_stored_fingerprint_is_replaced_on_its_own_line() {
        let after = marked(
            "translated_from:\n  ja: ffffffffffff\n  de: eeeeeeeeeeee\n",
            JAPANESE,
        );

        assert_eq!(
            after,
            note("translated_from:\n  ja: 0123456789ab\n  de: eeeeeeeeeeee\n")
        );
    }

    #[test]
    fn an_empty_entry_is_filled() {
        let after = marked("translated_from:\n  ja: ffffffffffff\n  de:\n", GERMAN);

        assert_eq!(
            after,
            note("translated_from:\n  ja: ffffffffffff\n  de: 0123456789ab\n")
        );
    }

    #[test]
    fn a_missing_german_entry_goes_after_the_japanese_one() {
        let after = marked("translated_from:\n  ja: ffffffffffff\n", GERMAN);

        assert_eq!(
            after,
            note("translated_from:\n  ja: ffffffffffff\n  de: 0123456789ab\n")
        );
    }

    #[test]
    fn a_missing_japanese_entry_goes_before_the_german_one() {
        let after = marked("translated_from:\n  de: eeeeeeeeeeee\n", JAPANESE);

        assert_eq!(
            after,
            note("translated_from:\n  ja: 0123456789ab\n  de: eeeeeeeeeeee\n")
        );
    }

    #[test]
    fn a_block_without_entries_gets_the_entry() {
        let after = marked("translated_from:\n", GERMAN);

        assert_eq!(after, note("translated_from:\n  de: 0123456789ab\n"));
    }

    #[test]
    fn the_indentation_of_the_block_is_kept() {
        let after = marked("translated_from:\n    ja: ffffffffffff\n", GERMAN);

        assert_eq!(
            after,
            note("translated_from:\n    ja: ffffffffffff\n    de: 0123456789ab\n")
        );
    }

    #[test]
    fn a_missing_block_is_written_before_sources() {
        let after = marked("", GERMAN);

        assert_eq!(after, note("translated_from:\n  de: 0123456789ab\n"));
    }

    #[test]
    fn a_missing_block_is_written_before_reviewed_when_there_are_no_sources() {
        let text = "---\ntype: skill\nname: Go\nreviewed: false\n---\n";

        let after = set(text, JAPANESE, &fingerprint()).unwrap();

        assert_eq!(
            after,
            "---\ntype: skill\nname: Go\ntranslated_from:\n  ja: 0123456789ab\nreviewed: false\n---\n"
        );
    }

    #[test]
    fn only_a_key_at_the_start_of_a_line_counts() {
        let text = "---\ntype: skill\nnotes:\n  translated_from: x\n  sources: y\ntranslated_from_old: z\n\
                    reviewed: false\n---\n";

        let after = set(text, JAPANESE, &fingerprint()).unwrap();

        assert_eq!(
            after,
            "---\ntype: skill\nnotes:\n  translated_from: x\n  sources: y\ntranslated_from_old: z\n\
             translated_from:\n  ja: 0123456789ab\nreviewed: false\n---\n"
        );
    }

    #[test]
    fn the_body_and_the_closing_line_stay_as_they_are() {
        let text = "---\nreviewed: false\n---\n\n## Description\n\nText  \n  with spaces.\n\n---\n";

        let after = set(text, JAPANESE, &fingerprint()).unwrap();

        assert_eq!(
            after,
            "---\ntranslated_from:\n  ja: 0123456789ab\nreviewed: false\n---\n\n## Description\n\nText  \n  with spaces.\n\n---\n"
        );
    }

    #[test]
    fn a_closing_line_at_the_end_of_the_file_stays_without_its_newline() {
        let after = set("---\nreviewed: false\n---", GERMAN, &fingerprint()).unwrap();

        assert_eq!(
            after,
            "---\ntranslated_from:\n  de: 0123456789ab\nreviewed: false\n---"
        );
    }

    #[test]
    fn a_value_on_the_key_line_is_refused() {
        for written in ["translated_from: {}\n", "translated_from: null\n"] {
            let result = set(&note(written), JAPANESE, &fingerprint());

            assert_eq!(result, Err(EditError::NotABlock), "{written:?}");
        }
    }

    #[test]
    fn a_note_without_a_field_to_go_before_is_refused() {
        let result = set("---\ntype: skill\n---\n", JAPANESE, &fingerprint());

        assert_eq!(result, Err(EditError::NoFieldAfter));
    }

    #[test]
    fn a_note_without_frontmatter_is_refused() {
        let result = set("type: skill\n", JAPANESE, &fingerprint());

        assert!(matches!(result, Err(EditError::NoFrontmatter(_))));
    }
}
