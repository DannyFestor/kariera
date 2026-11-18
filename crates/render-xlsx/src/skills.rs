use kariera_documents::{SkillGroup, SkillSheet, SkillSheetLabels};
use rust_xlsxwriter::XlsxError;

use crate::canvas::Canvas;
use crate::grid::{Columns, Slot};
use crate::look::Look;

const HEADER: [Slot; 4] = [
    Slot::new(Columns::new(0, 2), Look::Key),
    Slot::new(Columns::new(3, 6), Look::Key),
    Slot::new(Columns::new(7, 8), Look::Key),
    Slot::new(Columns::new(9, 11), Look::Key),
];
const CATEGORY: Slot = HEADER[0].looking(Look::SkillCategory);
const SKILL: Slot = HEADER[1].looking(Look::Body);
const EXPERIENCE: Slot = HEADER[2].looking(Look::BodyRight);
const LAST_USE: Slot = HEADER[3].looking(Look::BodySmall);

pub(crate) fn write(canvas: &mut Canvas<'_>, sheet: &SkillSheet) -> Result<(), XlsxError> {
    write_header(canvas, &sheet.labels)?;
    for group in &sheet.skills {
        write_group(canvas, group)?;
    }
    Ok(())
}

fn write_header(canvas: &mut Canvas<'_>, labels: &SkillSheetLabels) -> Result<(), XlsxError> {
    let titles = [
        labels.category,
        labels.skill,
        labels.experience,
        labels.last_use,
    ];
    for (slot, title) in HEADER.into_iter().zip(titles) {
        canvas.put(slot, title)?;
    }
    canvas.next_row();
    Ok(())
}

/// The category is one cell over all of its Skills' rows.
fn write_group(canvas: &mut Canvas<'_>, group: &SkillGroup) -> Result<(), XlsxError> {
    canvas.put_down(
        canvas.last_row_of(group.skills.len()),
        CATEGORY,
        &group.category,
    )?;
    for skill in &group.skills {
        canvas.put(SKILL, &skill.name)?;
        canvas.put(EXPERIENCE, &skill.experience)?;
        canvas.put(LAST_USE, &skill.last_use)?;
        canvas.next_row();
    }
    Ok(())
}
