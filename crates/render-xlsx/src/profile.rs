use kariera_documents::{QualificationEntry, SkillSheet};
use rust_xlsxwriter::XlsxError;

use crate::canvas::Canvas;
use crate::grid::{Columns, Slot};
use crate::look::Look;
use crate::text::bulleted;

const LABEL: Slot = Slot::new(Columns::new(0, 1), Look::Key);
const VALUE: Slot = Slot::new(Columns::new(2, 11), Look::Body);

pub(crate) fn write(canvas: &mut Canvas<'_>, sheet: &SkillSheet) -> Result<(), XlsxError> {
    let labels = &sheet.labels;
    let urls: Vec<&str> = [&sheet.portfolio_url, &sheet.github_url]
        .into_iter()
        .filter_map(|url| url.as_deref())
        .collect();
    let facts = [
        (labels.headline, sheet.headline.clone()),
        (labels.business, sheet.business.clone()),
        (
            labels.availability,
            sheet
                .availability
                .clone()
                .unwrap_or_else(|| labels.unknown.to_owned()),
        ),
        (labels.url, urls.join("\n")),
        (labels.certificates, dated(&sheet.certificates)),
        (labels.education, dated(&sheet.education)),
        (labels.strengths, bulleted(&sheet.strengths)),
    ];
    for (label, value) in facts {
        canvas.put(LABEL, label)?;
        canvas.put(VALUE, &value)?;
        canvas.next_row();
    }
    Ok(())
}

fn dated(entries: &[QualificationEntry]) -> String {
    entries
        .iter()
        .map(|entry| format!("{} {}", entry.date, entry.title))
        .collect::<Vec<_>>()
        .join("\n")
}
