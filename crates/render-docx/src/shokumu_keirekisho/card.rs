//! A Project's full card: a one-cell table with shading and a rule at the left.

use docx_rs::{
    BorderType, Paragraph, Shading, SpecialIndentType, Tab, TabValueType, Table, TableCellBorder,
    TableCellBorderPosition, TableCellMargins,
};
use kariera_documents::{PhaseMark, ProjectCard, ShokumuKeirekishoLabels};

use super::PALETTE;
use crate::blocks::{
    Block, body_paragraphs, cell, no_borders, right_tab, row, spacing, tab, table, text,
};
use crate::lists::bullet;
use crate::theme::{FAINT, MUTED, TEXT_WIDTH};

const CARD_FILL: &str = "F7F7F7";
const INDUSTRY_FILL: &str = "E1E1E1";
const SIDE_PADDING: usize = 200;
const INNER_WIDTH: usize = TEXT_WIDTH - 2 * SIDE_PADDING;
const LABEL_WIDTH: usize = 1650;
const FACT_SIZE: usize = 17;

pub(super) fn card(labels: &ShokumuKeirekishoLabels, project: &ProjectCard) -> Table {
    let mut blocks: Vec<Block> = vec![header(project).into()];
    blocks.extend(facts(labels, project).into_iter().map(Block::from));
    blocks.extend(
        body_paragraphs(&project.description)
            .into_iter()
            .map(Block::from),
    );
    blocks.extend(achievements(labels, project));
    let accent_rule = no_borders().set(
        TableCellBorder::new(TableCellBorderPosition::Left)
            .border_type(BorderType::Single)
            .size(16)
            .color(PALETTE.accent),
    );
    table(
        &[TEXT_WIDTH],
        vec![
            row(vec![
                cell(TEXT_WIDTH, accent_rule, blocks).shading(Shading::new().fill(CARD_FILL)),
            ])
            .cant_split(),
        ],
        TableCellMargins::new().margin(140, SIDE_PADDING, 140, SIDE_PADDING),
    )
}

fn header(project: &ProjectCard) -> Paragraph {
    Paragraph::new()
        .add_run(text(&project.period).bold().color(PALETTE.accent))
        .add_run(text("　"))
        .add_run(text(&project.title).bold())
        .add_run(tab())
        .add_run(
            text(&format!(" {} ", project.industry))
                .size(16)
                .shading(Shading::new().fill(INDUSTRY_FILL)),
        )
        .add_tab(right_tab(INNER_WIDTH))
        .keep_next(true)
        .line_spacing(spacing(0, 80))
}

fn facts(labels: &ShokumuKeirekishoLabels, project: &ProjectCard) -> Vec<Paragraph> {
    let role = project.focus.as_ref().map_or_else(
        || project.role.clone(),
        |focus| format!("{}（{focus}）", project.role),
    );
    let mut lines = vec![fact(labels.role, vec![text(&role)])];
    lines.extend(
        project
            .team
            .iter()
            .map(|team| fact(labels.team, vec![text(team)])),
    );
    lines.extend(
        project
            .end_client
            .iter()
            .map(|end_client| fact(labels.end_client, vec![text(end_client)])),
    );
    lines.push(fact(labels.phases, phase_runs(&project.phases)));
    lines.push(fact(
        labels.environment,
        vec![text(&project.skills.join(" / "))],
    ));
    lines
}

/// The label, a tab, then the value, which wraps under itself rather than under the label.
fn fact(label: &str, value: Vec<docx_rs::Run>) -> Paragraph {
    let line = Paragraph::new()
        .add_run(text(label).size(FACT_SIZE).color(MUTED))
        .add_run(tab())
        .add_tab(Tab::new().val(TabValueType::Left).pos(LABEL_WIDTH))
        .indent(
            Some(i32::try_from(LABEL_WIDTH).unwrap_or_default()),
            Some(SpecialIndentType::Hanging(
                i32::try_from(LABEL_WIDTH).unwrap_or_default(),
            )),
            None,
            None,
        )
        .line_spacing(spacing(0, 40));
    value
        .into_iter()
        .map(|run| run.size(FACT_SIZE))
        .fold(line, Paragraph::add_run)
}

fn phase_runs(phases: &[PhaseMark]) -> Vec<docx_rs::Run> {
    phases
        .iter()
        .map(|phase| {
            if phase.done {
                text(&format!("●{}　", phase.label))
                    .bold()
                    .color(PALETTE.accent)
            } else {
                text(&format!("○{}　", phase.label)).color(FAINT)
            }
        })
        .collect()
}

fn achievements(labels: &ShokumuKeirekishoLabels, project: &ProjectCard) -> Vec<Block> {
    if project.achievements.is_empty() {
        return Vec::new();
    }
    let title = Paragraph::new()
        .add_run(text(labels.achievements).bold().color(PALETTE.accent))
        .keep_next(true)
        .line_spacing(spacing(100, 40));
    std::iter::once(title)
        .chain(project.achievements.iter().map(|item| bullet(item)))
        .map(Block::from)
        .collect()
}
