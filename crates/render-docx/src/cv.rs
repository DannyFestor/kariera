//! The CV, layout variant C "Projects first" (issue #7), from the same model as the PDF.

mod band;
mod career;
mod glance;
mod profile;
mod projects;
mod qualifications;
mod skills;

use docx_rs::{Docx, Footer, Pic};
use kariera_documents::Cv;

use crate::RenderError;
use crate::blocks::{Block, add_blocks, spacer};
use crate::furniture::text_and_page_number;
use crate::heading::{HeadingLook, heading, with_heading_style};
use crate::lists::with_bullets;
use crate::package::{Page, base};
use crate::photo::picture;
use crate::theme::{Palette, Typeface};

const BODY_SIZE: usize = 18;
const PALETTE: Palette = Palette {
    accent: "C2410C",
    ink: "18181B",
};
const BAND: &str = "232A3B";
const HEADLINE_ON_BAND: &str = "FDBA74";
const HEADING: HeadingLook = HeadingLook {
    size: 28,
    color: PALETTE.ink,
    rule: None,
};

pub(crate) fn render(cv: &Cv) -> Result<Docx, RenderError> {
    let photo = cv.photo.as_ref().map(picture).transpose()?;
    let docx = base(
        Typeface::CV,
        BODY_SIZE,
        &Page {
            top: 794,
            bottom: 907,
        },
    );
    let docx = with_bullets(with_heading_style(docx, HEADING), PALETTE.accent);
    let footer = Footer::new().add_paragraph(text_and_page_number(&format!(
        "{} · {}",
        cv.name, cv.headline
    )));
    Ok(add_blocks(docx.footer(footer), body(cv, photo)))
}

fn body(cv: &Cv, photo: Option<Pic>) -> Vec<Block> {
    let mut blocks: Vec<Block> = vec![band::band(cv, photo).into(), spacer(8).into()];
    blocks.extend(glance::strip(cv));
    blocks.extend(profile::section(cv));
    blocks.extend(projects::section(cv));
    blocks.extend(career::section(cv));
    blocks.extend(skills::section(cv));
    blocks.extend(qualifications::section(cv));
    blocks
}

fn section_heading(title: &str) -> Block {
    heading(title, HEADING).into()
}
