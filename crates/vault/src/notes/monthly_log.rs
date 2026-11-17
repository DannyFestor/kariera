use kariera_domain::MonthlyLog;

use crate::fields::Fields;
use crate::problems::Reported;
use crate::sections::Sections;

/// Schema §2.5 and §1.11.
pub(super) fn read(
    fields: &mut Fields<'_>,
    sections: &mut Sections<'_>,
) -> Result<MonthlyLog, Reported> {
    let engagement = fields.required("engagement");
    let month = fields.required("month");
    let hours = fields.required("hours");
    let project_hours = fields.optional("project_hours");
    let sources = fields.required("sources");
    let focus = sections.one_line("Focus");
    let outcomes = sections.english_bullets("Outcomes");
    Ok(MonthlyLog {
        engagement: engagement?,
        month: month?,
        hours: hours?,
        project_hours: project_hours?.unwrap_or_default(),
        sources: sources?,
        focus: focus?,
        outcomes: outcomes?,
    })
}
