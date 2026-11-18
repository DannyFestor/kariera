use kariera_documents::{PhaseMark, SheetProject, SkillSheet, SkillSheetLabels, Technology};
use rust_xlsxwriter::XlsxError;

use crate::canvas::Canvas;
use crate::grid::{Columns, Slot};
use crate::look::Look;
use crate::text::bullet;

const TITLE_BAND_HEIGHT_POINTS: f64 = 20.0;
const NUMBER: Slot = Slot::new(Columns::one(0), Look::ProjectNumber);
const TITLE: Slot = Slot::new(Columns::new(1, 7), Look::ProjectTitle);
const PERIOD: Slot = Slot::new(Columns::new(8, 11), Look::ProjectPeriod);
const TECHNOLOGY_CATEGORY: Slot = Slot::new(Columns::new(0, 1), Look::TechnologyLabel);
const TECHNOLOGY_SKILLS: Slot = Slot::new(Columns::new(2, 11), Look::Fact);
const HEADING: Slot = Slot::new(Columns::FULL, Look::SectionLabel);
const PARAGRAPH: Slot = Slot::new(Columns::FULL, Look::Fact);
const PHASE_PAIR: Columns = Columns::new(0, 1);

/// A label and its value, side by side; three of them make a row of facts.
#[derive(Clone, Copy)]
struct FactSlot {
    label: Slot,
    value: Slot,
}

impl FactSlot {
    const fn at(label_column: u16) -> Self {
        Self {
            label: Slot::new(Columns::one(label_column), Look::FactLabel),
            value: Slot::new(Columns::new(label_column + 1, label_column + 3), Look::Fact),
        }
    }
}

const FIRST_SLOT: FactSlot = FactSlot::at(0);
const SECOND_SLOT: FactSlot = FactSlot::at(4);
const THIRD_SLOT: FactSlot = FactSlot::at(8);

/// Each Project is a block that the page breaks keep on one page.
pub(crate) fn write_all(canvas: &mut Canvas<'_>, sheet: &SkillSheet) -> Result<(), XlsxError> {
    for project in &sheet.projects {
        canvas.next_row();
        canvas.block(|canvas| write(canvas, &sheet.labels, project))?;
    }
    Ok(())
}

fn write(
    canvas: &mut Canvas<'_>,
    labels: &SkillSheetLabels,
    project: &SheetProject,
) -> Result<(), XlsxError> {
    write_title_band(canvas, labels, project)?;
    write_facts(canvas, labels, project)?;
    write_phases(canvas, &project.phases)?;
    write_technology(canvas, &project.technology)?;
    write_prose(canvas, labels.description, &project.description)?;
    let achievements: Vec<String> = project
        .achievements
        .iter()
        .map(|achievement| bullet(achievement))
        .collect();
    write_prose(canvas, labels.achievements, &achievements)
}

fn write_title_band(
    canvas: &mut Canvas<'_>,
    labels: &SkillSheetLabels,
    project: &SheetProject,
) -> Result<(), XlsxError> {
    canvas.put(NUMBER, &format!("{} {}", labels.number, project.number))?;
    canvas.put(TITLE, &project.title)?;
    canvas.put(
        PERIOD,
        &format!("{}（{}）", project.period, project.duration),
    )?;
    canvas.set_height(TITLE_BAND_HEIGHT_POINTS);
    canvas.next_row();
    Ok(())
}

fn write_facts(
    canvas: &mut Canvas<'_>,
    labels: &SkillSheetLabels,
    project: &SheetProject,
) -> Result<(), XlsxError> {
    let client = project.end_client.as_ref().map_or_else(
        || project.client.clone(),
        |end_client| format!("{} ／ {end_client}", project.client),
    );
    let role = project.focus.as_ref().map_or_else(
        || project.role.clone(),
        |focus| format!("{}（{focus}）", project.role),
    );
    let team = project.team.as_deref().unwrap_or(labels.unknown);
    write_fact_row(
        canvas,
        &[
            (FIRST_SLOT, labels.company, &client),
            (SECOND_SLOT, labels.industry, &project.industry),
            (THIRD_SLOT, labels.contract, &project.contract_type),
        ],
    )?;
    write_fact_row(
        canvas,
        &[
            (FIRST_SLOT, labels.role, &role),
            (SECOND_SLOT, labels.team, team),
        ],
    )
}

fn write_fact_row(
    canvas: &mut Canvas<'_>,
    facts: &[(FactSlot, &str, &str)],
) -> Result<(), XlsxError> {
    for &(slot, label, value) in facts {
        canvas.put(slot.label, label)?;
        canvas.put(slot.value, value)?;
    }
    canvas.next_row();
    Ok(())
}

/// Two columns to a phase, so the six phases fill the row.
fn write_phases(canvas: &mut Canvas<'_>, phases: &[PhaseMark]) -> Result<(), XlsxError> {
    for (first, phase) in (0..).step_by(2).zip(phases) {
        let look = if phase.done {
            Look::PhaseDone
        } else {
            Look::PhasePending
        };
        let columns = Columns::new(PHASE_PAIR.first + first, PHASE_PAIR.last + first);
        canvas.put(Slot::new(columns, look), &phase.label)?;
    }
    canvas.next_row();
    Ok(())
}

fn write_technology(canvas: &mut Canvas<'_>, technology: &[Technology]) -> Result<(), XlsxError> {
    for group in technology {
        canvas.put(TECHNOLOGY_CATEGORY, &group.category)?;
        canvas.put(TECHNOLOGY_SKILLS, &group.skills.join("、"))?;
        canvas.next_row();
    }
    Ok(())
}

/// A bracketed heading over one row per paragraph; nothing when there is no text.
fn write_prose(
    canvas: &mut Canvas<'_>,
    heading: &str,
    paragraphs: &[String],
) -> Result<(), XlsxError> {
    if paragraphs.is_empty() {
        return Ok(());
    }
    canvas.put(HEADING, &format!("【{heading}】"))?;
    canvas.next_row();
    for paragraph in paragraphs {
        canvas.put(PARAGRAPH, paragraph)?;
        canvas.next_row();
    }
    Ok(())
}
