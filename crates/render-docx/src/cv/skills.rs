//! Skills: a two-column table, the category and then its Skills with the years.

use docx_rs::{Paragraph, TableCellMargins};
use kariera_documents::{Cv, SkillGroup};

use super::section_heading;
use crate::blocks::{Block, paragraph_cell, row, table, text};
use crate::theme::{MUTED, TEXT_WIDTH};

const SIZE: usize = 17;
const CATEGORY_WIDTH: usize = 1800;
const SEPARATOR: &str = "   ·   ";

pub(super) fn section(cv: &Cv) -> Vec<Block> {
    if cv.skills.is_empty() {
        return Vec::new();
    }
    let rows = cv
        .skills
        .iter()
        .map(|group| {
            row(vec![
                paragraph_cell(CATEGORY_WIDTH, vec![category(group)]),
                paragraph_cell(TEXT_WIDTH - CATEGORY_WIDTH, vec![skills(group)]),
            ])
        })
        .collect();
    vec![
        section_heading(cv.labels.skills),
        table(
            &[CATEGORY_WIDTH, TEXT_WIDTH - CATEGORY_WIDTH],
            rows,
            TableCellMargins::new().margin(50, 60, 50, 0),
        )
        .into(),
    ]
}

fn category(group: &SkillGroup) -> Paragraph {
    Paragraph::new().add_run(text(&group.category).size(SIZE).bold())
}

fn skills(group: &SkillGroup) -> Paragraph {
    group
        .skills
        .iter()
        .enumerate()
        .flat_map(|(index, skill)| {
            let separator = (index > 0).then(|| text(SEPARATOR).size(SIZE).color(MUTED));
            separator.into_iter().chain([
                text(&skill.name).size(SIZE),
                text(&format!(" {}", skill.experience))
                    .size(SIZE)
                    .color(MUTED),
            ])
        })
        .fold(Paragraph::new(), Paragraph::add_run)
}
