//! Career: a four-column table with bottom borders only.

use docx_rs::{Paragraph, TableCellMargins, TableRow};
use kariera_documents::{CareerEntry, Cv};

use super::section_heading;
use crate::blocks::{Block, bottom_rule, cell, row, table, text};
use crate::theme::{MUTED, TEXT_WIDTH};

const SIZE: usize = 17;
const WIDTHS: [usize; 4] = [1900, 3300, 2900, TEXT_WIDTH - 1900 - 3300 - 2900];

pub(super) fn section(cv: &Cv) -> Vec<Block> {
    if cv.career.is_empty() {
        return Vec::new();
    }
    let rows = cv.career.iter().map(entry_row).collect();
    vec![
        section_heading(cv.labels.career),
        table(&WIDTHS, rows, TableCellMargins::new().margin(60, 60, 60, 0)).into(),
    ]
}

fn entry_row(entry: &CareerEntry) -> TableRow {
    let engagement_type = entry.projects.as_ref().map_or_else(
        || entry.engagement_type.clone(),
        |projects| format!("{} · {projects}", entry.engagement_type),
    );
    let cells = [
        text(&entry.period).color(MUTED),
        text(&entry.role).bold(),
        text(&entry.organization),
        text(&engagement_type).color(MUTED),
    ];
    row(cells
        .into_iter()
        .zip(WIDTHS)
        .map(|(run, width)| {
            cell(
                width,
                bottom_rule(),
                vec![Paragraph::new().add_run(run.size(SIZE)).into()],
            )
        })
        .collect())
    .cant_split()
}
