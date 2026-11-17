use std::fmt;
use std::str::FromStr;

use crate::ValueError;

/// A single-line string, trimmed and not blank (schema §1.2).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Text(String);

impl Text {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Text {
    type Err = ValueError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.trim().is_empty() {
            Err(ValueError::Blank)
        } else if value.contains(['\n', '\r']) {
            Err(ValueError::MultiLine)
        } else if value.trim() != value {
            Err(ValueError::Untrimmed)
        } else {
            Ok(Self(value.to_owned()))
        }
    }
}

impl fmt::Display for Text {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A Text starting with `https://` (schema §1.2).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Url(Text);

impl Url {
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl FromStr for Url {
    type Err = ValueError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let text = value.parse::<Text>()?;
        match value.strip_prefix("https://") {
            Some(rest) if !rest.is_empty() => Ok(Self(text)),
            Some(_) | None => Err(ValueError::NotHttps),
        }
    }
}

// The quoted text, so a snapshot of a whole note stays readable.
impl fmt::Debug for Text {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl fmt::Debug for Url {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::{Text, Url};
    use crate::ValueError;

    #[test]
    fn a_single_trimmed_line_is_text() {
        let text = "Backend developer".parse::<Text>();

        assert_eq!(
            text.map(|text| text.to_string()),
            Ok("Backend developer".to_owned())
        );
    }

    #[test]
    fn text_and_urls_debug_as_their_quoted_text() {
        let text = "Acme".parse::<Text>().unwrap();
        let url = "https://acme.example".parse::<Url>().unwrap();

        assert_eq!(
            format!("{text:?} {url:?}"),
            "\"Acme\" \"https://acme.example\""
        );
    }

    #[test]
    fn blank_text_is_refused() {
        assert_eq!(" ".parse::<Text>(), Err(ValueError::Blank));
    }

    #[test]
    fn text_with_a_line_break_is_refused() {
        assert_eq!("one\ntwo".parse::<Text>(), Err(ValueError::MultiLine));
        assert_eq!("one\rtwo".parse::<Text>(), Err(ValueError::MultiLine));
    }

    #[test]
    fn untrimmed_text_is_refused() {
        assert_eq!(" padded".parse::<Text>(), Err(ValueError::Untrimmed));
        assert_eq!("padded ".parse::<Text>(), Err(ValueError::Untrimmed));
    }

    #[test]
    fn an_https_address_is_a_url() {
        let url = "https://example.com".parse::<Url>();

        assert_eq!(url.as_ref().map(Url::as_str), Ok("https://example.com"));
    }

    #[test]
    fn a_url_without_https_is_refused() {
        assert_eq!(
            "http://example.com".parse::<Url>(),
            Err(ValueError::NotHttps)
        );
        assert_eq!("https://".parse::<Url>(), Err(ValueError::NotHttps));
    }

    #[test]
    fn a_url_is_text_first() {
        assert_eq!("https://a b\n".parse::<Url>(), Err(ValueError::MultiLine));
    }
}
