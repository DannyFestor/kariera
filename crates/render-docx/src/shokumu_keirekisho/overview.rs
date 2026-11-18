//! The strengths and the qualifications side by side, a two-column table without borders.

use docx_rs::{Paragraph, TableCellMargins};
use kariera_documents::{QualificationEntry, ShokumuKeirekisho, SpokenLanguage};

use super::section_heading;
use crate::blocks::{Block, plain_cell, row, spacing, table, text};
use crate::lists::bullet;
use crate::theme::{MUTED, TEXT_WIDTH};

const HALF: usize = TEXT_WIDTH / 2;

pub(super) fn section(document: &ShokumuKeirekisho) -> Vec<Block> {
    let strengths = std::iter::once(section_heading(document.labels.strengths))
        .chain(
            document
                .strengths
                .iter()
                .map(|strength| Block::from(bullet(strength))),
        )
        .collect();
    let qualifications = std::iter::once(section_heading(document.labels.qualifications))
        .chain(qualification_lines(document).into_iter().map(Block::from))
        .collect();
    vec![
        table(
            &[HALF, HALF],
            vec![row(vec![
                plain_cell(HALF, strengths),
                plain_cell(HALF, qualifications),
            ])],
            TableCellMargins::new().margin(0, 200, 0, 0),
        )
        .into(),
    ]
}

fn qualification_lines(document: &ShokumuKeirekisho) -> Vec<Paragraph> {
    let qualifications = document.qualifications.iter().map(qualification_line);
    let languages = document
        .languages
        .iter()
        .map(|language| language_line(document.labels.language, language));
    qualifications.chain(languages).collect()
}

fn qualification_line(qualification: &QualificationEntry) -> Paragraph {
    labelled_line(&qualification.date, &qualification.title)
}

fn language_line(label: &str, language: &SpokenLanguage) -> Paragraph {
    let value = language.level.as_ref().map_or_else(
        || language.name.clone(),
        |level| format!("{}（{level}）", language.name),
    );
    labelled_line(label, &value)
}

fn labelled_line(label: &str, value: &str) -> Paragraph {
    Paragraph::new()
        .add_run(text(label).color(MUTED))
        .add_run(text("　"))
        .add_run(text(value))
        .line_spacing(spacing(0, 20))
}
