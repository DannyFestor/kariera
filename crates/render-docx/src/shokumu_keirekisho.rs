//! The 職務経歴書, layout variant B「サマリー先行・カード型」 (issue #7), from the same model as the PDF. The duration of
//! a Skill is text where the PDF draws a bar.

mod card;
mod history;
mod older;
mod overview;
mod skills;
mod title;

use docx_rs::{Docx, Footer, Header};
use kariera_documents::ShokumuKeirekisho;

use crate::blocks::{Block, add_blocks, body_paragraphs};
use crate::furniture::{page_number_at_right, text_and_text};
use crate::heading::{HeadingLook, heading, with_heading_style};
use crate::lists::with_bullets;
use crate::package::{Page, base};
use crate::theme::{Palette, Typeface};

const BODY_SIZE: usize = 18;
const PALETTE: Palette = Palette {
    accent: "1F3A5F",
    ink: "000000",
};
const HEADING: HeadingLook = HeadingLook {
    size: 22,
    color: PALETTE.accent,
    rule: Some(PALETTE.accent),
};

pub(crate) fn render(document: &ShokumuKeirekisho) -> Docx {
    let docx = base(
        Typeface::JAPANESE,
        BODY_SIZE,
        &Page {
            top: 1021,
            bottom: 1021,
        },
    );
    let docx = with_bullets(with_heading_style(docx, HEADING), PALETTE.accent);
    add_blocks(with_page_furniture(docx, document), body(document))
}

/// The running header is left off the first page, which has its own title.
fn with_page_furniture(docx: Docx, document: &ShokumuKeirekisho) -> Docx {
    let running_title = format!("{}　{}", document.labels.title, document.name);
    docx.title_pg()
        .header(Header::new().add_paragraph(text_and_text(&running_title, &document.as_of)))
        .footer(page_footer())
        .first_footer(page_footer())
}

fn page_footer() -> Footer {
    Footer::new().add_paragraph(page_number_at_right())
}

fn body(document: &ShokumuKeirekisho) -> Vec<Block> {
    let mut blocks = title::block(document);
    blocks.push(section_heading(document.labels.summary));
    blocks.extend(
        body_paragraphs(&document.summary)
            .into_iter()
            .map(Block::from),
    );
    blocks.extend(overview::section(document));
    blocks.extend(skills::section(document));
    blocks.extend(history::section(document));
    blocks.extend(closing(document));
    blocks
}

fn section_heading(title: &str) -> Block {
    heading(title, HEADING).into()
}

fn closing(document: &ShokumuKeirekisho) -> Vec<Block> {
    let mut blocks = Vec::new();
    if !document.self_pr.is_empty() {
        blocks.push(section_heading(document.labels.self_pr));
        blocks.extend(
            body_paragraphs(&document.self_pr)
                .into_iter()
                .map(Block::from),
        );
    }
    blocks.extend(title::links(document));
    blocks.push(title::closing_mark(document).into());
    blocks
}
