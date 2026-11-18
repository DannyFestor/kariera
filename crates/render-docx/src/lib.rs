//! Renders document models to DOCX with docx-rs: real paragraphs, lists and tables that an agency can edit in Word or
//! `LibreOffice`, never images of text. The fonts are named, not embedded (`theme::Typeface`). The same document always
//! gives the same bytes: the file carries no clock, and nothing numbered from a counter shared by the process.
#![expect(
    clippy::multiple_crate_versions,
    reason = "docx-rs's zip pulls miniz_oxide 0.9 beside the 0.8 Typst's png uses (deny.toml lists them under `skip`)"
)]

mod blocks;
mod cv;
mod field;
mod furniture;
mod heading;
mod link;
mod lists;
mod package;
mod photo;
mod shokumu_keirekisho;
mod theme;

use kariera_documents::{Document, DocumentKind};
use kariera_domain::Language;
use thiserror::Error;

use package::Properties;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("the Skill Sheet is an XLSX document and has no DOCX layout")]
    NoLayout,
    #[error("cannot read the photo (PNG, JPEG, GIF and WebP are supported in DOCX): {0}")]
    Photo(image::ImageError),
    #[error("cannot write the DOCX file: {0}")]
    Pack(String),
}

/// The documents with a DOCX layout.
#[must_use]
pub const fn renders(kind: DocumentKind) -> bool {
    match kind {
        DocumentKind::Cv | DocumentKind::ShokumuKeirekisho => true,
        DocumentKind::SkillSheet => false,
    }
}

pub fn render(document: &Document) -> Result<Vec<u8>, RenderError> {
    match document {
        Document::Cv(cv) => package::bytes(
            cv::render(cv)?,
            &Properties {
                title: cv.labels.title,
                author: &cv.name,
                language: cv.language,
            },
        ),
        Document::ShokumuKeirekisho(document) => package::bytes(
            shokumu_keirekisho::render(document),
            &Properties {
                title: document.labels.title,
                author: &document.name,
                language: Language::Japanese.code(),
            },
        ),
        Document::SkillSheet(_) => Err(RenderError::NoLayout),
    }
}
