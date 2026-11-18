//! Selected projects: per Project a bold title with the period at the right margin, a muted facts line, the
//! description, the achievements as a list and the Skills as one line of text.

use docx_rs::Paragraph;
use kariera_documents::{Cv, CvLabels, CvProject};

use super::{PALETTE, section_heading};
use crate::blocks::{Block, keep_together, right_tab, spacing, tab, text};
use crate::lists::bullet;
use crate::theme::{MUTED, TEXT_WIDTH};

const TITLE_SIZE: usize = 21;
const SMALL: usize = 16;
const FACT_SEPARATOR: &str = "  ·  ";

pub(super) fn section(cv: &Cv) -> Vec<Block> {
    if cv.projects.is_empty() {
        return Vec::new();
    }
    let heading = section_heading(cv.labels.selected_projects);
    let projects = cv
        .projects
        .iter()
        .flat_map(|project| project_paragraphs(project, &cv.labels))
        .map(Block::from);
    std::iter::once(heading).chain(projects).collect()
}

fn project_paragraphs(project: &CvProject, labels: &CvLabels) -> Vec<Paragraph> {
    let mut paragraphs = vec![title_line(project), facts_line(project, labels)];
    paragraphs.extend(project.description.iter().map(|description| {
        Paragraph::new()
            .add_run(text(description))
            .line_spacing(spacing(0, 60))
    }));
    paragraphs.extend(project.achievements.iter().map(|item| bullet(item)));
    paragraphs.push(skills_line(project));
    keep_together(paragraphs)
}

fn title_line(project: &CvProject) -> Paragraph {
    Paragraph::new()
        .add_run(text(&project.title).size(TITLE_SIZE).bold())
        .add_run(tab())
        .add_run(
            text(&project.period)
                .size(SMALL)
                .bold()
                .color(PALETTE.accent),
        )
        .add_tab(right_tab(TEXT_WIDTH))
        .line_spacing(spacing(160, 0))
}

fn facts_line(project: &CvProject, labels: &CvLabels) -> Paragraph {
    let role = project.focus.as_ref().map_or_else(
        || project.role.clone(),
        |focus| format!("{} ({focus})", project.role),
    );
    let facts = [
        Some(role),
        Some(format!("{} {}", labels.client, project.client)),
        project
            .end_client
            .as_ref()
            .map(|end_client| format!("{} {end_client}", labels.end_client)),
        Some(project.industry.clone()),
        project.team.clone(),
    ];
    let line: Vec<String> = facts.into_iter().flatten().collect();
    Paragraph::new()
        .add_run(text(&line.join(FACT_SEPARATOR)).size(SMALL).color(MUTED))
        .line_spacing(spacing(0, 60))
}

fn skills_line(project: &CvProject) -> Paragraph {
    Paragraph::new()
        .add_run(
            text(&project.skills.join(FACT_SEPARATOR))
                .size(SMALL)
                .color(MUTED),
        )
        .line_spacing(spacing(40, 60))
}
