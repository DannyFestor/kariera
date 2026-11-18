//! Education and Certificates: a two-column table without borders.

use docx_rs::{Paragraph, TableCellMargins};
use kariera_documents::{Cv, QualificationEntry};

use super::section_heading;
use crate::blocks::{Block, plain_cell, row, spacing, table, text};
use crate::theme::{MUTED, TEXT_WIDTH};

const HALF: usize = TEXT_WIDTH / 2;

pub(super) fn section(cv: &Cv) -> Vec<Block> {
    if cv.education.is_empty() && cv.certificates.is_empty() {
        return Vec::new();
    }
    let columns = vec![
        column(cv.labels.education, &cv.education),
        column(cv.labels.certificates, &cv.certificates),
    ];
    let cells = columns
        .into_iter()
        .map(|blocks| plain_cell(HALF, blocks))
        .collect();
    vec![
        table(
            &[HALF, HALF],
            vec![row(cells)],
            TableCellMargins::new().margin(0, 200, 0, 0),
        )
        .into(),
    ]
}

fn column(title: &str, entries: &[QualificationEntry]) -> Vec<Block> {
    if entries.is_empty() {
        return vec![Paragraph::new().into()];
    }
    std::iter::once(section_heading(title))
        .chain(entries.iter().flat_map(entry).map(Block::from))
        .collect()
}

fn entry(entry: &QualificationEntry) -> [Paragraph; 2] {
    let details = entry.issuer.as_ref().map_or_else(
        || entry.date.clone(),
        |issuer| format!("{issuer} · {}", entry.date),
    );
    [
        Paragraph::new()
            .add_run(text(&entry.title).bold())
            .keep_next(true)
            .line_spacing(spacing(0, 0)),
        Paragraph::new()
            .add_run(text(&details).color(MUTED))
            .line_spacing(spacing(0, 80)),
    ]
}
