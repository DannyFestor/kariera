//! Whether a string needs YAML quotes, for the schema's rule that strings are written plain unless YAML requires
//! quotes (§1.2).

use crate::yaml::{self, Style, Value};

/// True when `text` written plain would not be read back as the same string: it is not valid plain YAML, or YAML
/// reads it as `null`, a boolean or a number.
#[must_use]
pub fn requires_quotes(text: &str) -> bool {
    !is_read_back_unchanged(text) || is_null_boolean_or_number(text)
}

fn is_read_back_unchanged(text: &str) -> bool {
    let Ok(Some(root)) = yaml::parse(&format!("key: {text}\n"), 1) else {
        return false;
    };
    matches!(
        root.get("key").map(|node| &node.value),
        Some(Value::Scalar { text: read, style: Style::Plain }) if read == text
    )
}

/// The YAML 1.2 core schema, which the parser follows.
fn is_null_boolean_or_number(text: &str) -> bool {
    matches!(
        text,
        "" | "~"
            | "null"
            | "Null"
            | "NULL"
            | "true"
            | "True"
            | "TRUE"
            | "false"
            | "False"
            | "FALSE"
    ) || is_number(text)
}

fn is_number(text: &str) -> bool {
    let unsigned = text.strip_prefix(['-', '+']).unwrap_or(text);
    matches!(unsigned, ".inf" | ".Inf" | ".INF")
        || matches!(text, ".nan" | ".NaN" | ".NAN")
        || is_prefixed_integer(text)
        || is_decimal(unsigned)
}

fn is_prefixed_integer(text: &str) -> bool {
    let digits_of = |prefix: &str, radix: u32| {
        text.strip_prefix(prefix)
            .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_digit(radix)))
    };
    digits_of("0x", 16) || digits_of("0o", 8)
}

/// `digits`, `digits.digits`, `digits.` or `.digits`, then an optional exponent.
fn is_decimal(unsigned: &str) -> bool {
    let (mantissa, exponent) = unsigned
        .split_once(['e', 'E'])
        .map_or((unsigned, None), |(mantissa, exponent)| {
            (mantissa, Some(exponent))
        });
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let all_digits = |text: &str| text.bytes().all(|byte| byte.is_ascii_digit());
    let has_digit = !whole.is_empty() || !fraction.is_empty();
    has_digit && all_digits(whole) && all_digits(fraction) && exponent.is_none_or(is_exponent)
}

fn is_exponent(exponent: &str) -> bool {
    let digits = exponent.strip_prefix(['-', '+']).unwrap_or(exponent);
    !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::requires_quotes;

    #[test]
    fn ordinary_text_is_written_plain() {
        for text in [
            "Lazuli",
            "Backend developer",
            "バックエンド開発者",
            "150 h",
            "2024-05",
            "N2",
            "980/990",
            "Node.js",
            "Shibuya, Tokyo",
            "a-b",
            "https://example.com",
            "yes",
        ] {
            assert!(!requires_quotes(text), "{text}");
        }
    }

    #[test]
    fn text_yaml_would_read_differently_needs_quotes() {
        for text in [
            "",
            " lead",
            "trail ",
            "a: b",
            "a #b",
            "- a",
            "? a",
            "[a]",
            "{a}",
            "@a",
            "&a",
            "*a",
            "!a",
            "|a",
            ">a",
            "%a",
            "'a",
            "\"a",
            "a: ",
            "#a",
            "owner: said so later",
        ] {
            assert!(requires_quotes(text), "{text:?}");
        }
    }

    #[test]
    fn text_that_looks_like_null_a_boolean_or_a_number_needs_quotes() {
        for text in [
            "null",
            "Null",
            "NULL",
            "~",
            "true",
            "True",
            "FALSE",
            "980",
            "-5",
            "+5",
            "1.5",
            "1.",
            ".5",
            "1e3",
            "1.5E-3",
            "0x1F",
            "0o17",
            ".inf",
            "-.INF",
            ".nan",
            "1234567890123",
        ] {
            assert!(requires_quotes(text), "{text:?}");
        }
    }

    #[test]
    fn text_that_only_resembles_a_number_is_plain() {
        for text in [
            "1e", "e3", "1.2.3", "0x", "0xg", "0o8", "--5", ".", "-.", "1e+", "+.nan", "12:30",
            "1_000",
        ] {
            assert!(!requires_quotes(text), "{text:?}");
        }
    }
}
