use kariera_domain::Language;

/// The documents kariera builds. The 履歴書 is not among them yet: its layout is still open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DocumentKind {
    Cv,
    ShokumuKeirekisho,
    SkillSheet,
}

/// A document in one language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Edition {
    pub kind: DocumentKind,
    pub language: Language,
}

impl DocumentKind {
    pub const ALL: [Self; 3] = [Self::Cv, Self::ShokumuKeirekisho, Self::SkillSheet];

    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Cv => "cv",
            Self::ShokumuKeirekisho => "shokumu-keirekisho",
            Self::SkillSheet => "skill-sheet",
        }
    }

    /// The languages the document is written in: the CV is for readers outside Japan, the others for Japanese
    /// readers.
    #[must_use]
    pub const fn languages(self) -> &'static [Language] {
        match self {
            Self::Cv => &[Language::English, Language::German],
            Self::ShokumuKeirekisho | Self::SkillSheet => &[Language::Japanese],
        }
    }

    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.slug() == slug)
    }
}

impl Edition {
    /// Every document in every language it is written in.
    #[must_use]
    pub fn all() -> Vec<Self> {
        DocumentKind::ALL
            .into_iter()
            .flat_map(|kind| {
                kind.languages()
                    .iter()
                    .map(move |&language| Self { kind, language })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use kariera_domain::Language;

    use super::{DocumentKind, Edition};

    #[test]
    fn the_cv_is_english_and_german_and_the_japanese_documents_are_japanese() {
        let editions: Vec<(&str, &str)> = Edition::all()
            .into_iter()
            .map(|edition| (edition.kind.slug(), edition.language.code()))
            .collect();

        assert_eq!(
            editions,
            [
                ("cv", "en"),
                ("cv", "de"),
                ("shokumu-keirekisho", "ja"),
                ("skill-sheet", "ja")
            ]
        );
    }

    #[test]
    fn a_document_is_found_by_its_slug() {
        assert_eq!(
            DocumentKind::from_slug("skill-sheet"),
            Some(DocumentKind::SkillSheet)
        );
        assert_eq!(DocumentKind::from_slug("rirekisho"), None);
        assert!(DocumentKind::Cv.languages().contains(&Language::German));
    }
}
