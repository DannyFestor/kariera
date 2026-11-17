use std::fmt;
use std::str::FromStr;

use crate::ValueError;

/// What `translated_from` holds for one language (schema §1.6.1): 12 lowercase hex digits.
///
/// They are the start of a digest of the note's English translatable content, made as ADR 0002 says.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fingerprint(String);

const HEX_RADIX: u32 = 16;
const BITS_PER_DIGIT: u32 = 4;

impl Fingerprint {
    pub const BYTES: usize = 6;
    const LENGTH: usize = Self::BYTES * 2;

    #[must_use]
    pub fn from_bytes(bytes: [u8; Self::BYTES]) -> Self {
        let digits = bytes.into_iter().flat_map(|byte| {
            [byte >> BITS_PER_DIGIT, byte & 0x0f]
                .into_iter()
                .filter_map(|digit| char::from_digit(u32::from(digit), HEX_RADIX))
        });
        Self(digits.collect())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Fingerprint {
    type Err = ValueError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let is_lowercase_hex = |digit: char| digit.is_ascii_digit() || ('a'..='f').contains(&digit);
        if value.chars().count() == Self::LENGTH && value.chars().all(is_lowercase_hex) {
            Ok(Self(value.to_owned()))
        } else {
            Err(ValueError::NotAFingerprint)
        }
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

// The quoted text, so a snapshot of a whole note stays readable.
impl fmt::Debug for Fingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::Fingerprint;
    use crate::ValueError;

    #[test]
    fn bytes_are_written_as_lowercase_hex() {
        let fingerprint = Fingerprint::from_bytes([0x00, 0x0f, 0xa0, 0x3f, 0x9a, 0xff]);

        assert_eq!(fingerprint.to_string(), "000fa03f9aff");
    }

    #[test]
    fn twelve_lowercase_hex_digits_are_a_fingerprint() {
        let fingerprint = "3f9a1c0b7e2d".parse::<Fingerprint>();

        assert_eq!(
            fingerprint.as_ref().map(Fingerprint::as_str),
            Ok("3f9a1c0b7e2d")
        );
    }

    #[test]
    fn anything_else_is_refused() {
        for text in [
            "",
            "3f9a1c0b7e2",
            "3f9a1c0b7e2d0",
            "3F9A1C0B7E2D",
            "3f9a1c0b7e2g",
            "3f9a1c0b7e２d",
        ] {
            assert_eq!(
                text.parse::<Fingerprint>(),
                Err(ValueError::NotAFingerprint),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_fingerprint_debugs_as_its_quoted_text() {
        let fingerprint = "3f9a1c0b7e2d".parse::<Fingerprint>().unwrap();

        assert_eq!(format!("{fingerprint:?}"), "\"3f9a1c0b7e2d\"");
    }
}
