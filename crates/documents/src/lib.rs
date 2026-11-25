//! Format-independent: the render crates turn these models into PDF, DOCX and XLSX.
//!
//! A model holds the document's content in its language, already chosen and written out: which Projects and Skills
//! it shows, every text in the language with the English where a translation is missing, and dates and lengths of
//! time as the language writes them. Headings and layout belong to the renderers.

mod audience;
mod context;
mod cv;
mod edition;
mod experience;
mod labels;
mod locale;
mod months;
mod parts;
mod rirekisho;
mod shokumu_keirekisho;
mod skill_sheet;
#[cfg(test)]
mod test_career;
mod translator;

use thiserror::Error;

pub use audience::Audience;
pub use context::Inputs;
pub use cv::{CareerEntry, Contact, Cv, CvProject, PersonalDetails};
pub use edition::{DocumentKind, Edition};
pub use labels::{CvLabels, RirekishoLabels, ShokumuKeirekishoLabels, SkillSheetLabels};
pub use parts::{PhaseMark, QualificationEntry, SkillEntry, SkillGroup, SpokenLanguage};
pub use rirekisho::{Address, HistoryLine, Residence, Rirekisho};
pub use shokumu_keirekisho::{EngagementHistory, ProjectCard, ProjectLine, ShokumuKeirekisho};
pub use skill_sheet::{SheetProject, SkillSheet, Technology};
pub use translator::MissingTranslation;

use context::Context;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Document {
    Cv(Cv),
    ShokumuKeirekisho(ShokumuKeirekisho),
    Rirekisho(Rirekisho),
    SkillSheet(SkillSheet),
}

impl Document {
    /// The document's title in its language and the owner's name, such as `Lebenslauf_Jane_Example`; a renderer
    /// adds its extension.
    #[must_use]
    pub fn file_stem(&self) -> String {
        match self {
            Self::Cv(cv) => file_stem(cv.labels.title, &cv.name),
            Self::ShokumuKeirekisho(document) => file_stem(document.labels.title, &document.name),
            Self::Rirekisho(document) => file_stem(document.labels.title, &document.name),
            Self::SkillSheet(sheet) => file_stem(sheet.labels.title, &sheet.name),
        }
    }
}

fn file_stem(title: &str, name: &str) -> String {
    let name: String = name
        .chars()
        .map(|character| {
            if character.is_whitespace() {
                '_'
            } else {
                character
            }
        })
        .collect();
    format!("{title}_{name}")
}

/// A document, and the notes whose English it printed for want of a translation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Built {
    pub document: Document,
    pub missing_translations: Vec<MissingTranslation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DocumentError {
    #[error("the vault has no Profile, which every document starts from")]
    NoProfile,
}

pub fn build(edition: Edition, inputs: &Inputs<'_>) -> Result<Built, DocumentError> {
    let context = Context::new(inputs, edition).ok_or(DocumentError::NoProfile)?;
    let document = match edition.kind {
        DocumentKind::Cv => Document::Cv(cv::build(&context)),
        DocumentKind::ShokumuKeirekisho => {
            Document::ShokumuKeirekisho(shokumu_keirekisho::build(&context))
        }
        DocumentKind::Rirekisho => Document::Rirekisho(rirekisho::build(&context)),
        DocumentKind::SkillSheet => Document::SkillSheet(skill_sheet::build(&context)),
    };
    Ok(Built {
        document,
        missing_translations: context.into_missing(),
    })
}

#[cfg(test)]
mod tests {
    use super::file_stem;

    #[test]
    fn a_file_stem_is_the_title_and_the_name_with_underscores_for_spaces() {
        assert_eq!(
            file_stem("Lebenslauf", "Jane Example"),
            "Lebenslauf_Jane_Example"
        );
        assert_eq!(
            file_stem("職務経歴書", "Jane　Example"),
            "職務経歴書_Jane_Example"
        );
    }
}
