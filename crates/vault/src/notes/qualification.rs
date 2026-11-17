use kariera_domain::Qualification;

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
    let issuer = fields.optional("issuer");
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
