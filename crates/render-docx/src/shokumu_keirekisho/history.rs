//! 職務経歴: per Engagement a heading, then a card for each recent Project and a ruled table for the older ones.

use docx_rs::Paragraph;
use kariera_documents::{EngagementHistory, ShokumuKeirekisho};

use super::{card, older, section_heading};
use crate::blocks::{Block, right_tab, spacer, spacing, tab, text};
use crate::theme::{MUTED, TEXT_WIDTH};

pub(super) fn section(document: &ShokumuKeirekisho) -> Vec<Block> {
    let heading = section_heading(document.labels.history);
    let note = Paragraph::new()
        .add_run(text(&document.history_note).size(16).color(MUTED))
        .keep_next(true);
    let engagements = document
        .engagements
        .iter()
        .flat_map(|engagement| engagement_blocks(document, engagement));
    [heading, note.into()]
        .into_iter()
        .chain(engagements)
        .collect()
}

fn engagement_blocks(document: &ShokumuKeirekisho, engagement: &EngagementHistory) -> Vec<Block> {
    let mut blocks: Vec<Block> = heading_lines(document, engagement)
        .into_iter()
        .map(Block::from)
        .collect();
    for project in &engagement.cards {
        blocks.push(card::card(&document.labels, project).into());
        blocks.push(spacer(120).into());
    }
    if !engagement.older.is_empty() {
        blocks.push(older::projects_table(&engagement.older).into());
        blocks.push(spacer(120).into());
    }
    blocks
}

fn heading_lines(document: &ShokumuKeirekisho, engagement: &EngagementHistory) -> Vec<Paragraph> {
    let labels = &document.labels;
    let mut lines = vec![
        Paragraph::new()
            .add_run(text(&engagement.organization).size(21).bold())
            .add_run(text(&format!("　{}", engagement.period)).color(MUTED))
            .add_run(tab())
            .add_run(text(&engagement.engagement_type))
            .add_tab(right_tab(TEXT_WIDTH))
            .line_spacing(spacing(200, 20)),
    ];
    lines.extend(
        engagement
            .role
            .iter()
            .map(|role| detail(&format!("{}：{role}", labels.position), false)),
    );
    if !engagement.business.is_empty() {
        let business = engagement.business.join(" ");
        lines.push(detail(&format!("{}：{business}", labels.business), true));
    }
    lines.into_iter().map(|line| line.keep_next(true)).collect()
}

fn detail(content: &str, muted: bool) -> Paragraph {
    let run = text(content).size(17);
    Paragraph::new().add_run(if muted { run.color(MUTED) } else { run })
}
