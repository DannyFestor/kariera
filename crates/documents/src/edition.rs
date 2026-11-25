use kariera_domain::Language;

use crate::Audience;

/// The documents kariera builds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DocumentKind {
    Cv,
    ShokumuKeirekisho,
    Rirekisho,
    SkillSheet,
}

/// A document in one language, for one audience.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Edition {
    pub kind: DocumentKind,
    pub language: Language,
    pub audience: Audience,
}

impl DocumentKind {
    pub const ALL: [Self; 4] = [
        Self::Cv,
        Self::ShokumuKeirekisho,
        Self::Rirekisho,
        Self::SkillSheet,
    ];

    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Cv => "cv",
            Self::ShokumuKeirekisho => "shokumu-keirekisho",
            Self::Rirekisho => "rirekisho",
            Self::SkillSheet => "skill-sheet",
        }
    }

    /// The languages the document is written in: the CV is for readers outside Japan, the others for Japanese
    /// readers.
    #[must_use]
    pub const fn languages(self) -> &'static [Language] {
        match self {
            Self::Cv => &[Language::English, Language::German],
            Self::ShokumuKeirekisho | Self::Rirekisho | Self::SkillSheet => &[Language::Japanese],
        }
    }

    /// The owner's website shows only the CV and the 職務経歴書; the 履歴書 and the Skill Sheet go to applications.
    #[must_use]
    pub const fn audiences(self) -> &'static [Audience] {
        match self {
            Self::Cv | Self::ShokumuKeirekisho => &[Audience::Private, Audience::Public],
            Self::Rirekisho | Self::SkillSheet => &[Audience::Private],
        }
    }

    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.slug() == slug)
    }
}

impl Edition {
    /// Every document in every language it is written in, every private edition before every public one.
    #[must_use]
    pub fn all() -> Vec<Self> {
        Audience::ALL
            .into_iter()
            .flat_map(|audience| {
                DocumentKind::ALL
                    .into_iter()
                    .filter(move |kind| kind.audiences().contains(&audience))
                    .flat_map(move |kind| Self::of(kind, audience))
            })
            .collect()
    }

    fn of(kind: DocumentKind, audience: Audience) -> impl Iterator<Item = Self> {
        kind.languages().iter().map(move |&language| Self {
            kind,
            language,
            audience,
        })
    }
}

#[cfg(test)]
mod tests {
    use kariera_domain::Language;

    use super::{DocumentKind, Edition};
    use crate::Audience;

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
                ("rirekisho", "ja"),
                ("skill-sheet", "ja"),
                ("cv", "en"),
                ("cv", "de"),
                ("shokumu-keirekisho", "ja"),
            ]
        );
    }

    #[test]
    fn the_public_editions_are_the_cv_and_the_shokumu_keirekisho_after_every_private_one() {
        let audiences: Vec<(Audience, &str)> = Edition::all()
            .into_iter()
            .map(|edition| (edition.audience, edition.kind.slug()))
            .collect();

        assert_eq!(
            audiences[5..],
            [
                (Audience::Public, "cv"),
                (Audience::Public, "cv"),
                (Audience::Public, "shokumu-keirekisho"),
            ]
        );
        assert!(
            audiences[..5]
                .iter()
                .all(|&(audience, _)| audience == Audience::Private)
        );
    }

    #[test]
    fn only_the_cv_and_the_shokumu_keirekisho_have_a_public_edition() {
        let public: Vec<DocumentKind> = DocumentKind::ALL
            .into_iter()
            .filter(|kind| kind.audiences().contains(&Audience::Public))
            .collect();

        assert_eq!(public, [DocumentKind::Cv, DocumentKind::ShokumuKeirekisho]);
        assert!(
            DocumentKind::ALL
                .into_iter()
                .all(|kind| kind.audiences().contains(&Audience::Private))
        );
    }

    #[test]
    fn a_document_is_found_by_its_slug() {
        assert_eq!(
            DocumentKind::from_slug("skill-sheet"),
            Some(DocumentKind::SkillSheet)
        );
        assert_eq!(
            DocumentKind::from_slug("rirekisho"),
            Some(DocumentKind::Rirekisho)
        );
        assert_eq!(DocumentKind::from_slug("rirekisyo"), None);
        assert!(DocumentKind::Cv.languages().contains(&Language::German));
    }
}
