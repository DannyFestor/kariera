use kariera_domain::{Issuer, Qualification, QualificationCategory};

use crate::fields::Fields;
use crate::problems::Reported;
use crate::sections::Sections;

/// Schema §2.7.
pub(super) fn read(
    fields: &mut Fields<'_>,
    _empty_body: &mut Sections<'_>,
) -> Result<Qualification, Reported> {
    let name = fields.required("name");
    let labels = fields.translations("labels");
    let category = fields.required("category");
    let issuer = issuer(fields, category);
    let start = fields.optional("start");
    let date = fields.required("date");
    let level = fields.optional("level");
    let score = fields.optional("score");
    let translated_from = fields.translations("translated_from");
    let sources = fields.required("sources");
    let reviewed = fields.required("reviewed");
    Ok(Qualification {
        name: name?,
        labels: labels?,
        category: category?,
        issuer: issuer?,
        start: start?,
        date: date?,
        level: level?,
        score: score?,
        translated_from: translated_from?,
        sources: sources?,
        reviewed: reviewed?,
    })
}

/// A link to a school for a degree, Text for every other category; not checked when the category could not be read.
fn issuer(
    fields: &mut Fields<'_>,
    category: Result<QualificationCategory, Reported>,
) -> Result<Option<Issuer>, Reported> {
    match category {
        Ok(QualificationCategory::Degree) => fields
            .optional("issuer")
            .map(|school| school.map(Issuer::School)),
        Ok(
            QualificationCategory::Certificate
            | QualificationCategory::Licence
            | QualificationCategory::TestResult,
        ) => fields.optional("issuer").map(|body| body.map(Issuer::Body)),
        Err(Reported) => {
            fields.skip("issuer");
            Err(Reported)
        }
    }
}
