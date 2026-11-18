//! Headers and footers with Word's page number fields, which Word and `LibreOffice` count themselves.

use docx_rs::{AlignmentType, Paragraph, Run};

use crate::blocks::{right_tab, tab, text};
use crate::field::field;
use crate::theme::{MUTED, TEXT_WIDTH};

const SIZE: usize = 15;

fn muted(content: &str) -> Run {
    text(content).size(SIZE).color(MUTED)
}

fn with_runs(paragraph: Paragraph, runs: Vec<Run>) -> Paragraph {
    runs.into_iter().fold(paragraph, Paragraph::add_run)
}

/// `1 / 3`: the page and the number of pages.
fn page_of_pages(paragraph: Paragraph) -> Paragraph {
    let paragraph = with_runs(paragraph, field("PAGE", muted("1")));
    let paragraph = paragraph.add_run(muted(" / "));
    with_runs(paragraph, field("NUMPAGES", muted("1")))
}

pub(crate) fn page_number_at_right() -> Paragraph {
    page_of_pages(Paragraph::new()).align(AlignmentType::Right)
}

pub(crate) fn text_and_page_number(left: &str) -> Paragraph {
    let line = Paragraph::new()
        .add_run(muted(left))
        .add_run(tab())
        .add_tab(right_tab(TEXT_WIDTH));
    page_of_pages(line)
}

pub(crate) fn text_and_text(left: &str, right: &str) -> Paragraph {
    Paragraph::new()
        .add_run(muted(left))
        .add_run(tab())
        .add_run(muted(right))
        .add_tab(right_tab(TEXT_WIDTH))
}
