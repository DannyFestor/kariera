//! The title block at the top of page 1, and the portfolio links and 以上 at the end.

use docx_rs::{AlignmentType, Paragraph, Run};
use kariera_documents::ShokumuKeirekisho;

use super::PALETTE;
use crate::blocks::{Block, right_tab, spacing, tab, text};
use crate::link::hyperlink;
use crate::theme::{MUTED, TEXT_WIDTH};

pub(super) fn block(document: &ShokumuKeirekisho) -> Vec<Block> {
    let title = Paragraph::new()
        .add_run(
            text(document.labels.title)
                .size(40)
                .bold()
                .color(PALETTE.accent),
        )
        .line_spacing(spacing(0, 0));
    let name = Paragraph::new()
        .add_run(text(&document.name).size(24).bold())
        .add_run(text("　"))
        .add_run(text(&document.headline).color(MUTED))
        .add_run(tab())
        .add_run(text(&document.as_of).size(17).color(MUTED))
        .add_tab(right_tab(TEXT_WIDTH))
        .line_spacing(spacing(0, 120));
    vec![title.into(), name.into()]
}

pub(super) fn links(document: &ShokumuKeirekisho) -> Vec<Block> {
    let portfolio = document
        .portfolio_url
        .iter()
        .map(|url| (document.labels.portfolio, url));
    let github = document
        .github_url
        .iter()
        .map(|url| (document.labels.github, url));
    let runs: Vec<Run> = portfolio
        .chain(github)
        .flat_map(|(label, url)| labelled_link(label, url))
        .collect();
    if runs.is_empty() {
        return Vec::new();
    }
    let line = runs
        .into_iter()
        .fold(Paragraph::new(), Paragraph::add_run)
        .line_spacing(spacing(120, 0));
    vec![line.into()]
}

fn labelled_link(label: &str, url: &str) -> Vec<Run> {
    let mut runs = vec![text(&format!("{label}："))];
    runs.extend(hyperlink(url, url, PALETTE.accent));
    runs.push(text("　"));
    runs
}

pub(super) fn closing_mark(document: &ShokumuKeirekisho) -> Paragraph {
    Paragraph::new()
        .add_run(text(document.labels.closing))
        .align(AlignmentType::Right)
}
