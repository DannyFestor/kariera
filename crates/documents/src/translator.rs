//! Picks each text in the document's language. A missing Japanese or German text prints the English one, and the
//! build reports the note and language (owner's decision on issue #7, provisional).

use std::cell::RefCell;
use std::collections::BTreeSet;

use kariera_domain::{
    Language, NoteName, Paragraphs, Text, Translated, TranslationLanguage, Translations,
};

/// A note that printed its English text in a document of another language.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MissingTranslation {
    pub note: NoteName,
    pub language: Language,
}

/// A text that may be empty: an empty English text needs no translation.
pub(crate) trait Content {
    fn is_blank(&self) -> bool;
}

impl Content for Text {
    fn is_blank(&self) -> bool {
        false
    }
}

impl Content for Paragraphs {
    fn is_blank(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl Content for Vec<Text> {
    fn is_blank(&self) -> bool {
        self.is_empty()
    }
}

#[derive(Debug)]
pub(crate) struct Translator {
    language: Language,
    missing: RefCell<BTreeSet<MissingTranslation>>,
}

impl Translator {
    pub(crate) const fn new(language: Language) -> Self {
        Self {
            language,
            missing: RefCell::new(BTreeSet::new()),
        }
    }

    pub(crate) fn pick<'value, T: Content>(
        &self,
        note: &NoteName,
        value: &'value Translated<T>,
    ) -> &'value T {
        value.get(self.language).unwrap_or_else(|| {
            if !value.en.is_blank() {
                self.missing.borrow_mut().insert(MissingTranslation {
                    note: note.clone(),
                    language: self.language,
                });
            }
            &value.en
        })
    }

    /// A `labels` entry, or `name` where there is none: an empty label means the name is used as it is, so it is
    /// not a missing translation.
    pub(crate) fn label<'value>(
        &self,
        name: &'value Text,
        labels: &'value Translations<Text>,
    ) -> &'value Text {
        self.translation_language()
            .and_then(|language| labels.get(language))
            .unwrap_or(name)
    }

    pub(crate) fn into_missing(self) -> Vec<MissingTranslation> {
        self.missing.into_inner().into_iter().collect()
    }

    const fn translation_language(&self) -> Option<TranslationLanguage> {
        match self.language {
            Language::English => None,
            Language::Japanese => Some(TranslationLanguage::Japanese),
            Language::German => Some(TranslationLanguage::German),
        }
    }
}

#[cfg(test)]
mod tests {
    use kariera_domain::{Language, NoteName, Paragraphs, Translated, Translations};

    use super::{MissingTranslation, Translator};
    use crate::test_career::text;

    fn note(name: &str) -> NoteName {
        name.parse().unwrap()
    }

    #[test]
    fn a_translated_text_is_printed_in_the_document_s_language() {
        let translator = Translator::new(Language::German);
        let role = Translated {
            en: text("Developer"),
            ja: None,
            de: Some(text("Entwickler")),
        };

        assert_eq!(translator.pick(&note("Shop"), &role), &text("Entwickler"));
        assert_eq!(translator.into_missing(), []);
    }

    #[test]
    fn a_missing_translation_prints_the_english_and_names_the_note_and_language() {
        let translator = Translator::new(Language::Japanese);
        let role = Translated {
            en: text("Developer"),
            ja: None,
            de: Some(text("Entwickler")),
        };

        assert_eq!(translator.pick(&note("Shop"), &role), &text("Developer"));
        assert_eq!(translator.pick(&note("Shop"), &role), &text("Developer"));
        assert_eq!(
            translator.into_missing(),
            [MissingTranslation {
                note: note("Shop"),
                language: Language::Japanese
            }]
        );
    }

    #[test]
    fn an_empty_english_text_needs_no_translation() {
        let translator = Translator::new(Language::German);
        let achievements: Translated<Vec<_>> = Translated {
            en: Vec::new(),
            ja: None,
            de: None,
        };
        let description = Translated {
            en: Paragraphs(String::new()),
            ja: None,
            de: None,
        };

        translator.pick(&note("Shop"), &achievements);
        translator.pick(&note("Shop"), &description);

        assert_eq!(translator.into_missing(), []);
    }

    #[test]
    fn a_missing_label_prints_the_name_without_a_warning() {
        let translator = Translator::new(Language::Japanese);
        let labels = Translations {
            ja: None,
            de: Some(text("Bachelor Informatik")),
        };

        assert_eq!(translator.label(&text("BSc"), &labels), &text("BSc"));
        assert_eq!(translator.into_missing(), []);
    }

    #[test]
    fn a_label_is_printed_in_the_document_s_language_and_english_prints_the_name() {
        let labels = Translations {
            ja: Some(text("株式会社アクメ")),
            de: None,
        };

        let japanese = Translator::new(Language::Japanese);
        let english = Translator::new(Language::English);

        assert_eq!(
            japanese.label(&text("Acme"), &labels),
            &text("株式会社アクメ")
        );
        assert_eq!(english.label(&text("Acme"), &labels), &text("Acme"));
    }
}
