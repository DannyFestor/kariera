//! Renders document models to PDF with Typst embedded as a library: no `typst` binary, no system fonts, no network.
//! The same document always gives the same bytes.
#![expect(
    clippy::multiple_crate_versions,
    reason = "Typst 0.15.1 itself depends on two versions of several crates (deny.toml lists them under `skip`)"
)]

mod world;

use kariera_documents::Document;
use thiserror::Error;
use typst::diag::SourceDiagnostic;
use typst::ecow::EcoVec;
use typst::foundations::{Bytes, Dict, IntoValue};
use typst_layout::PagedDocument;
use typst_pdf::PdfOptions;

use world::EmbeddedWorld;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pdf {
    pub bytes: Vec<u8>,
    pub pages: usize,
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("cannot hand the {template} to Typst: {source}")]
    Data {
        template: &'static str,
        source: serde_json::Error,
    },
    #[error("Typst cannot lay out the {template}: {messages}")]
    Layout {
        template: &'static str,
        messages: String,
    },
    #[error("Typst cannot write the PDF of the {template}: {messages}")]
    Pdf {
        template: &'static str,
        messages: String,
    },
}

/// A template and what it reads from `sys.inputs`.
struct Job<'document> {
    name: &'static str,
    template: &'static str,
    data: Result<Vec<u8>, serde_json::Error>,
    photo: Option<&'document [u8]>,
}

pub fn render(document: &Document) -> Result<Pdf, RenderError> {
    let job = job(document);
    let template = job.name;
    let data = job
        .data
        .map_err(|source| RenderError::Data { template, source })?;
    let world = EmbeddedWorld::new(job.template, inputs(data, job.photo));
    let laid_out: PagedDocument =
        typst::compile(&world)
            .output
            .map_err(|diagnostics| RenderError::Layout {
                template,
                messages: messages(&diagnostics),
            })?;
    let bytes = typst_pdf::pdf(&laid_out, &PdfOptions::default()).map_err(|diagnostics| {
        RenderError::Pdf {
            template,
            messages: messages(&diagnostics),
        }
    })?;
    Ok(Pdf {
        bytes,
        pages: laid_out.pages().len(),
    })
}

fn job(document: &Document) -> Job<'_> {
    match document {
        Document::Cv(cv) => Job {
            name: "CV",
            template: include_str!("../templates/cv.typ"),
            data: serde_json::to_vec(cv),
            photo: cv.photo.as_ref().map(|photo| photo.0.as_slice()),
        },
        Document::ShokumuKeirekisho(shokumu_keirekisho) => Job {
            name: "職務経歴書",
            template: include_str!("../templates/shokumu-keirekisho.typ"),
            data: serde_json::to_vec(shokumu_keirekisho),
            photo: None,
        },
        Document::Rirekisho(rirekisho) => Job {
            name: "履歴書",
            template: include_str!("../templates/rirekisho.typ"),
            data: serde_json::to_vec(rirekisho),
            photo: rirekisho.photo.as_ref().map(|photo| photo.0.as_slice()),
        },
        Document::SkillSheet(skill_sheet) => Job {
            name: "Skill Sheet",
            template: include_str!("../templates/skill-sheet.typ"),
            data: serde_json::to_vec(skill_sheet),
            photo: None,
        },
    }
}

fn inputs(data: Vec<u8>, photo: Option<&[u8]>) -> Dict {
    let mut inputs = Dict::new();
    inputs.insert("data".into(), Bytes::new(data).into_value());
    if let Some(photo) = photo {
        inputs.insert("photo".into(), Bytes::new(photo.to_vec()).into_value());
    }
    inputs
}

fn messages(diagnostics: &EcoVec<SourceDiagnostic>) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("; ")
}
