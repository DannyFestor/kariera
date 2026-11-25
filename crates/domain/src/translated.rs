use crate::{Language, TranslationLanguage};

/// A value in English with its optional Japanese and German translations.
///
/// A Map3 field (schema §1.5), or a body section with its `## 日本語` and `## Deutsch` counterparts (§1.6). A missing
/// translation is not translated yet.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Translated<T> {
    pub en: T,
    pub ja: Option<T>,
    pub de: Option<T>,
}

/// The maps with only the keys `ja` and `de`: `labels` (schema §1.5) and `translated_from` (§1.6.1).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Translations<T> {
    pub ja: Option<T>,
    pub de: Option<T>,
}

impl<T> Translated<T> {
    #[must_use]
    pub const fn get(&self, language: Language) -> Option<&T> {
        match language {
            Language::English => Some(&self.en),
            Language::Japanese => self.ja.as_ref(),
            Language::German => self.de.as_ref(),
        }
    }

    /// For a value outside the vault, such as the nationality of `personal.yaml`: no translation is reported missing.
    #[must_use]
    pub fn get_or_english(&self, language: Language) -> &T {
        self.get(language).unwrap_or(&self.en)
    }
}

impl<T> Translations<T> {
    #[must_use]
    pub const fn none() -> Self {
        Self { ja: None, de: None }
    }

    #[must_use]
    pub const fn get(&self, language: TranslationLanguage) -> Option<&T> {
        match language {
            TranslationLanguage::Japanese => self.ja.as_ref(),
            TranslationLanguage::German => self.de.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Translated, Translations};
    use crate::{Language, TranslationLanguage};

    #[test]
    fn each_language_gets_its_own_value() {
        let role = Translated {
            en: "Developer",
            ja: Some("開発者"),
            de: Some("Entwickler"),
        };

        let values = Language::ALL.map(|language| role.get(language).copied());

        assert_eq!(
            values,
            [Some("Developer"), Some("開発者"), Some("Entwickler")]
        );
    }

    #[test]
    fn an_untranslated_language_has_no_value() {
        let role = Translated {
            en: "Developer",
            ja: None,
            de: None,
        };

        assert_eq!(role.get(Language::German), None);
    }

    #[test]
    fn an_untranslated_language_falls_back_to_english_on_request() {
        let nationality = Translated {
            en: "German",
            ja: Some("ドイツ"),
            de: None,
        };

        assert_eq!(*nationality.get_or_english(Language::Japanese), "ドイツ");
        assert_eq!(*nationality.get_or_english(Language::German), "German");
    }

    #[test]
    fn no_translations_hold_no_value() {
        assert_eq!(
            Translations::<u8>::none(),
            Translations { ja: None, de: None }
        );
    }

    #[test]
    fn each_translation_language_gets_its_own_value() {
        let labels = Translations {
            ja: Some("ゴー"),
            de: None,
        };

        let values = TranslationLanguage::ALL.map(|language| labels.get(language).copied());

        assert_eq!(values, [Some("ゴー"), None]);
    }
}
