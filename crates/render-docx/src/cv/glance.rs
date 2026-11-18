//! The at-a-glance strip: a shaded paragraph with availability, spoken languages and personal details.

use docx_rs::{Paragraph, Run};
use kariera_documents::{Cv, PersonalDetails, SpokenLanguage};

use super::PALETTE;
use crate::blocks::{Block, shaded, spacing, text};
use crate::theme::CHIP;

const SIZE: usize = 16;
const SEPARATOR: &str = "      ·      ";

pub(super) fn strip(cv: &Cv) -> Vec<Block> {
    let mut items: Vec<Run> = Vec::new();
    items.extend(
        cv.availability
            .iter()
            .map(|availability| text(availability).bold().color(PALETTE.accent)),
    );
    items.extend(languages(&cv.languages).map(|names| text(&names)));
    items.extend(
        cv.personal_details
            .iter()
            .filter_map(|details| personal_details(cv.labels.born, details))
            .map(|line| text(&line)),
    );
    if items.is_empty() {
        return Vec::new();
    }
    let line = separated(items)
        .into_iter()
        .map(|run| run.size(SIZE))
        .fold(Paragraph::new(), Paragraph::add_run)
        .line_spacing(spacing(60, 60));
    vec![shaded(line, CHIP).into()]
}

fn separated(items: Vec<Run>) -> Vec<Run> {
    let last = items.len().saturating_sub(1);
    items
        .into_iter()
        .enumerate()
        .flat_map(|(index, item)| {
            let separator = (index < last).then(|| text(SEPARATOR));
            std::iter::once(item).chain(separator)
        })
        .collect()
}

fn languages(languages: &[SpokenLanguage]) -> Option<String> {
    let names: Vec<String> = languages
        .iter()
        .map(|language| {
            language.level.as_ref().map_or_else(
                || language.name.clone(),
                |level| format!("{} ({level})", language.name),
            )
        })
        .collect();
    (!names.is_empty()).then(|| names.join(" · "))
}

fn personal_details(born: &str, details: &PersonalDetails) -> Option<String> {
    let parts: Vec<String> = details
        .birth_date
        .iter()
        .map(|date| format!("{born} {date}"))
        .chain(details.nationality.clone())
        .collect();
    (!parts.is_empty()).then(|| parts.join(" · "))
}
