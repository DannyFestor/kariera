use kariera_documents::SkillSheet;
use rust_xlsxwriter::XlsxError;

use crate::canvas::Canvas;
use crate::grid::{Columns, Slot};
use crate::look::Look;

const LABEL: Slot = Slot::new(Columns::new(0, 1), Look::Key);
const PARAGRAPH: Slot = Slot::new(Columns::new(2, 11), Look::Body);

/// One paragraph to a row, because a row is at most 409 points high.
pub(crate) fn write(canvas: &mut Canvas<'_>, sheet: &SkillSheet) -> Result<(), XlsxError> {
    if sheet.self_pr.is_empty() {
        return Ok(());
    }
    canvas.put_down(
        canvas.last_row_of(sheet.self_pr.len()),
        LABEL,
        sheet.labels.self_pr,
    )?;
    for paragraph in &sheet.self_pr {
        canvas.put(PARAGRAPH, paragraph)?;
        canvas.next_row();
    }
    Ok(())
}
