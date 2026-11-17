use kariera_vault::{Body, Line, Section};

/// A line of a note body with the heading of the section it is in, written as the vault reader writes headings:
/// `## Description` or `## 日本語 > ### Description`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BodyLine<'a> {
    pub(crate) heading: String,
    pub(crate) line: &'a Line,
}

pub(crate) fn body_lines(body: &Body) -> Vec<BodyLine<'_>> {
    let mut all = Vec::new();
    for section in &body.sections {
        all.extend(lines_under(&section_heading(section), section));
        for subsection in &section.subsections {
            let heading = subsection_heading(section, subsection);
            all.extend(lines_under(&heading, subsection));
        }
    }
    all
}

pub(crate) fn section_heading(section: &Section) -> String {
    format!("## {}", section.title)
}

pub(crate) fn subsection_heading(section: &Section, subsection: &Section) -> String {
    format!("## {} > ### {}", section.title, subsection.title)
}

fn lines_under<'a>(heading: &str, section: &'a Section) -> Vec<BodyLine<'a>> {
    section
        .lines
        .iter()
        .map(|line| BodyLine {
            heading: heading.to_owned(),
            line,
        })
        .collect()
}
