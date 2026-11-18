//! テクニカルスキル in three columns by category, each Skill with the time used as text.

use docx_rs::{Paragraph, TableCell, TableCellMargins};
use kariera_documents::{ShokumuKeirekisho, SkillGroup};

use super::{PALETTE, section_heading};
use crate::blocks::{Block, paragraph_cell, right_tab, row, spacing, tab, table, text};
use crate::theme::{MUTED, TEXT_WIDTH};

const COLUMNS: usize = 3;
const COLUMN_WIDTH: usize = TEXT_WIDTH / COLUMNS;
const RIGHT_PADDING: usize = 200;
const SIZE: usize = 16;

pub(super) fn section(document: &ShokumuKeirekisho) -> Vec<Block> {
    if document.skills.is_empty() {
        return Vec::new();
    }
    let rows = document
        .skills
        .chunks(COLUMNS)
        .map(|groups| row(columns(groups)))
        .collect();
    vec![
        section_heading(document.labels.skills),
        Paragraph::new()
            .add_run(text(&document.skills_note).size(15).color(MUTED))
            .keep_next(true)
            .into(),
        table(
            &[COLUMN_WIDTH; COLUMNS],
            rows,
            TableCellMargins::new().margin(0, RIGHT_PADDING, 120, 0),
        )
        .into(),
    ]
}

/// Always three cells: a row with fewer would stretch its cells over the width of the table.
fn columns(groups: &[SkillGroup]) -> Vec<TableCell> {
    let filled = groups.iter().map(category_cell);
    let empty =
        (groups.len()..COLUMNS).map(|_| paragraph_cell(COLUMN_WIDTH, vec![Paragraph::new()]));
    filled.chain(empty).collect()
}

fn category_cell(group: &SkillGroup) -> TableCell {
    let title = Paragraph::new()
        .add_run(
            text(&group.category)
                .size(SIZE)
                .bold()
                .color(PALETTE.accent),
        )
        .keep_next(true)
        .line_spacing(spacing(0, 40));
    let skills = group.skills.iter().map(|skill| {
        Paragraph::new()
            .add_run(text(&skill.name).size(SIZE))
            .add_run(tab())
            .add_run(text(&skill.experience).size(SIZE).color(MUTED))
            .add_tab(right_tab(COLUMN_WIDTH - RIGHT_PADDING))
            .line_spacing(spacing(0, 20))
    });
    paragraph_cell(COLUMN_WIDTH, std::iter::once(title).chain(skills).collect())
}
