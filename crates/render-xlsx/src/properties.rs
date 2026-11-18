use kariera_documents::SkillSheet;
use rust_xlsxwriter::{DocProperties, ExcelDateTime, XlsxError};

/// The same Skill Sheet gives the same bytes: `rust_xlsxwriter` would stamp the time of the build.
pub(crate) fn of(sheet: &SkillSheet) -> Result<DocProperties, XlsxError> {
    let fixed_time = ExcelDateTime::from_ymd(2000, 1, 1)?;
    Ok(DocProperties::new()
        .set_title(sheet.labels.title)
        .set_author(&sheet.name)
        .set_creation_datetime(&fixed_time))
}
