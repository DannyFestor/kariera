//! The translation fingerprint algorithm. Changing any step marks every translation stale, so it is fixed in
//! ADR 0002.

use kariera_domain::{EnglishText, Fingerprint};
use sha2::{Digest, Sha256};

use crate::names::nfc;

pub(crate) fn fingerprint(content: &[EnglishText]) -> Fingerprint {
    let canonical: String = content.iter().map(canonical_part).collect();
    let digest = Sha256::digest(canonical.as_bytes());
    let mut prefix = [0; Fingerprint::BYTES];
    for (slot, byte) in prefix.iter_mut().zip(digest) {
        *slot = byte;
    }
    Fingerprint::from_bytes(prefix)
}

/// The length makes the parts unambiguous: no text can imitate the start of the next part.
fn canonical_part(part: &EnglishText) -> String {
    let text = normalized(&part.text);
    format!("{}:{}:{text}\n", part.part, text.len())
}

/// Text that reads the same on every platform and in every editor is the same text: NFC (schema §1.1), LF line
/// endings, no trailing whitespace on a line, and no blank lines around the text.
fn normalized(text: &str) -> String {
    let text = nfc(&text.replace("\r\n", "\n").replace('\r', "\n"));
    let lines: Vec<&str> = text.lines().map(str::trim_end).collect();
    let first = lines.iter().position(|line| !line.is_empty());
    let last = lines.iter().rposition(|line| !line.is_empty());
    match (first, last) {
        (Some(first), Some(last)) => lines.get(first..=last).unwrap_or_default().join("\n"),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use kariera_domain::EnglishText;

    use super::fingerprint;

    fn content(parts: &[(&'static str, &str)]) -> Vec<EnglishText> {
        parts
            .iter()
            .map(|&(part, text)| EnglishText {
                part,
                text: text.to_owned(),
            })
            .collect()
    }

    fn fingerprint_of(parts: &[(&'static str, &str)]) -> String {
        fingerprint(&content(parts)).to_string()
    }

    // Each expected value is the first 12 digits of `printf '<canonical form>' | shasum -a 256` (ADR 0002).
    #[test]
    fn a_name_is_hashed_with_its_part_and_length() {
        assert_eq!(fingerprint_of(&[("name", "Go")]), "007a4f991fa0");
    }

    #[test]
    fn several_parts_are_hashed_in_order_with_their_lengths() {
        let parts = [("role", "Developer"), ("## Summary", "Built it.\n\nIn Go.")];

        assert_eq!(fingerprint_of(&parts), "742ef317c743");
    }

    #[test]
    fn text_is_hashed_in_nfc() {
        let composed = fingerprint_of(&[("name", "M\u{fc}ller")]);
        let decomposed = fingerprint_of(&[("name", "Mu\u{308}ller")]);

        assert_eq!(composed, "eab262440762");
        assert_eq!(decomposed, composed);
    }

    #[test]
    fn line_endings_trailing_whitespace_and_surrounding_blank_lines_do_not_matter() {
        let plain = fingerprint_of(&[("## Summary", "One.\n\nTwo.")]);

        for text in [
            "One.\r\n\r\nTwo.",
            "One.\r\rTwo.",
            "One. \t\n\nTwo.  ",
            "\n\nOne.\n\nTwo.\n\n",
        ] {
            assert_eq!(fingerprint_of(&[("## Summary", text)]), plain, "{text:?}");
        }
    }

    #[test]
    fn words_and_inner_spacing_matter() {
        let base = fingerprint_of(&[("## Summary", "Built it.")]);

        assert_ne!(fingerprint_of(&[("## Summary", "Built  it.")]), base);
        assert_ne!(fingerprint_of(&[("## Summary", " Built it.")]), base);
        assert_ne!(fingerprint_of(&[("## Summary", "Built it")]), base);
        assert_ne!(
            fingerprint_of(&[("## Summary", "Built it.\n\n\nMore.")]),
            fingerprint_of(&[("## Summary", "Built it.\n\nMore.")])
        );
    }

    #[test]
    fn the_same_text_in_another_part_is_another_fingerprint() {
        let role_then_focus = fingerprint_of(&[("role", "Developer"), ("focus", "Backend")]);
        let swapped = fingerprint_of(&[("role", "Backend"), ("focus", "Developer")]);

        assert_ne!(role_then_focus, swapped);
    }

    #[test]
    fn text_cannot_imitate_the_next_part() {
        let two_parts = fingerprint_of(&[("role", "A"), ("focus", "B")]);
        let one_part = fingerprint_of(&[("role", "A\nfocus:1:B")]);

        assert_ne!(two_parts, one_part);
    }

    #[test]
    fn no_content_has_a_fingerprint_too() {
        assert_eq!(fingerprint_of(&[]).len(), 12);
    }
}
