use std::str::FromStr;

use crate::ValueError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Language {
    English,
    Japanese,
    German,
}

impl Language {
    pub const ALL: [Self; 3] = [Self::English, Self::Japanese, Self::German];

    /// The key of the language in a translatable map (schema §1.5).
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Japanese => "ja",
            Self::German => "de",
        }
    }
}

/// The languages a note is translated into, which is every language but the English it is translated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TranslationLanguage {
    Japanese,
    German,
}

impl TranslationLanguage {
    pub const ALL: [Self; 2] = [Self::Japanese, Self::German];

    #[must_use]
    pub fn code(self) -> &'static str {
        Language::from(self).code()
    }
}

impl From<TranslationLanguage> for Language {
    fn from(language: TranslationLanguage) -> Self {
        match language {
            TranslationLanguage::Japanese => Self::Japanese,
            TranslationLanguage::German => Self::German,
        }
    }
}

impl FromStr for TranslationLanguage {
    type Err = ValueError;

    fn from_str(code: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|language| language.code() == code)
            .ok_or(ValueError::NotATranslationLanguage)
    }
}

#[cfg(test)]
mod tests {
    use super::{Language, TranslationLanguage};
    use crate::ValueError;

    #[test]
    fn codes_are_the_keys_of_a_translatable_map() {
        let codes = Language::ALL.map(Language::code);

        assert_eq!(codes, ["en", "ja", "de"]);
    }

    #[test]
    fn a_translation_language_is_read_from_its_code() {
        assert_eq!(
            "ja".parse::<TranslationLanguage>(),
            Ok(TranslationLanguage::Japanese)
        );
        assert_eq!(
            "de".parse::<TranslationLanguage>(),
            Ok(TranslationLanguage::German)
        );
    }

    #[test]
    fn english_is_not_a_translation_language() {
        assert_eq!(
            "en".parse::<TranslationLanguage>(),
            Err(ValueError::NotATranslationLanguage)
        );
    }
}
