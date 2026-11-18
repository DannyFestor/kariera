//! The older Projects, one row each in a table ruled between rows.

use docx_rs::{Paragraph, Table, TableCellMargins, TableRow};
use kariera_documents::ProjectLine;

use crate::blocks::{Block, bottom_rule, cell, row, table, text};
use crate::theme::{MUTED, TEXT_WIDTH};

const WIDTHS: [usize; 3] = [1360, TEXT_WIDTH - 1360 - 1930, 1930];

pub(super) fn projects_table(projects: &[ProjectLine]) -> Table {
    table(
        &WIDTHS,
        projects.iter().map(project_row).collect(),
        TableCellMargins::new().margin(50, 60, 50, 60),
    )
}

fn project_row(project: &ProjectLine) -> TableRow {
    let period = Paragraph::new().add_run(text(&project.period).size(16).color(MUTED));
    let summary = [
        Paragraph::new()
            .add_run(text(&project.title).bold())
            .add_run(
                text(&format!("（{}）", project.industry))
                    .size(15)
                    .color(MUTED),
            ),
        Paragraph::new().add_run(text(&project.skills.join(" / ")).size(15).color(MUTED)),
    ];
    let role = Paragraph::new().add_run(text(&project.role).size(16));
    let columns: [Vec<Block>; 3] = [
        vec![period.into()],
        summary.into_iter().map(Block::from).collect(),
        vec![role.into()],
    ];
    row(columns
        .into_iter()
        .zip(WIDTHS)
        .map(|(blocks, width)| cell(width, bottom_rule(), blocks))
        .collect())
    .cant_split()
}
