//! Profile: the summary beside the strengths, a two-column table without borders.

use docx_rs::{
    BorderType, Paragraph, TableCell, TableCellBorder, TableCellBorderPosition, TableCellMargins,
};
use kariera_documents::Cv;

use super::{PALETTE, section_heading};
use crate::blocks::{
    Block, body_paragraphs, cell, no_borders, paragraph_cell, row, spacing, table, text,
};
use crate::lists::bullet;
use crate::theme::TEXT_WIDTH;

const STRENGTHS_WIDTH: usize = 3600;

pub(super) fn section(cv: &Cv) -> Vec<Block> {
    let summary_width = TEXT_WIDTH - STRENGTHS_WIDTH;
    let summary = paragraph_cell(summary_width, body_paragraphs(&cv.summary));
    vec![
        section_heading(cv.labels.profile),
        table(
            &[summary_width, STRENGTHS_WIDTH],
            vec![row(vec![summary, strengths_cell(cv)])],
            TableCellMargins::new().margin(0, 200, 0, 200),
        )
        .into(),
    ]
}

fn strengths_cell(cv: &Cv) -> TableCell {
    if cv.strengths.is_empty() {
        return cell(STRENGTHS_WIDTH, no_borders(), vec![Paragraph::new().into()]);
    }
    let accent_rule = no_borders().set(
        TableCellBorder::new(TableCellBorderPosition::Left)
            .border_type(BorderType::Single)
            .size(12)
            .color(PALETTE.accent),
    );
    let title = Paragraph::new()
        .add_run(text(cv.labels.strengths).bold())
        .line_spacing(spacing(0, 40));
    let items = cv.strengths.iter().map(|strength| bullet(strength));
    let paragraphs = std::iter::once(title)
        .chain(items)
        .map(Block::from)
        .collect();
    cell(STRENGTHS_WIDTH, accent_rule, paragraphs)
}
