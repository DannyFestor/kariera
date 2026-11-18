use kariera_documents::SkillSheet;
use rust_xlsxwriter::XlsxError;

use crate::canvas::Canvas;
use crate::grid::{Columns, Slot};
use crate::look::Look;

const BAND_HEIGHT_POINTS: f64 = 24.0;
const TITLE: Slot = Slot::new(Columns::new(0, 8), Look::Band);
const UPDATED: Slot = Slot::new(Columns::new(9, 11), Look::BandRight);

pub(crate) fn write(canvas: &mut Canvas<'_>, sheet: &SkillSheet) -> Result<(), XlsxError> {
    let labels = &sheet.labels;
    canvas.put(
        TITLE,
        &format!("{}\u{3000}\u{3000}{}", labels.title, sheet.name),
    )?;
    canvas.put(UPDATED, &format!("{} {}", labels.updated, sheet.updated))?;
    canvas.set_height(BAND_HEIGHT_POINTS);
    canvas.next_row();
    Ok(())
}
