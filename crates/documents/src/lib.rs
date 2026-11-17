//! Format-independent: the render crates turn these models into PDF, DOCX and XLSX.
//!
//! A model holds the document's content in its language, already chosen and written out: which Projects and Skills
//! it shows, every text in the language with the English where a translation is missing, and dates and lengths of
//! time as the language writes them. Headings and layout belong to the renderers.

mod context;
mod cv;
mod edition;
mod experience;
mod locale;
mod months;
mod parts;
mod shokumu_keirekisho;
mod skill_sheet;
#[cfg(test)]
mod test_career;
mod translator;

use thiserror::Error;

pub use context::Inputs;
pub use cv::{Availability, CareerEntry, Contact, Cv, CvProject, PersonalDetails};
pub use edition::{DocumentKind, Edition};
pub use parts::{PhaseMark, QualificationEntry, SkillEntry, SkillGroup, SpokenLanguage};
pub use shokumu_keirekisho::{EngagementHistory, ProjectCard, ProjectLine, ShokumuKeirekisho};
pub use skill_sheet::{SheetProject, SkillSheet, Technology};
pub use translator::MissingTranslation;

use context::Context;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Document {
    Cv(Cv),
    ShokumuKeirekisho(ShokumuKeirekisho),
    SkillSheet(SkillSheet),
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
    let context = Context::new(inputs, edition.language).ok_or(DocumentError::NoProfile)?;
    let document = match edition.kind {
        DocumentKind::Cv => Document::Cv(cv::build(&context)),
        DocumentKind::ShokumuKeirekisho => {
            Document::ShokumuKeirekisho(shokumu_keirekisho::build(&context))
        }
        DocumentKind::SkillSheet => Document::SkillSheet(skill_sheet::build(&context)),
    };
    Ok(Built {
        document,
        missing_translations: context.into_missing(),
    })
}
