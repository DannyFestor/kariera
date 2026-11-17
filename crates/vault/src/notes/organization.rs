use kariera_domain::Organization;

use crate::fields::Fields;
use crate::problems::Reported;
use crate::sections::Sections;

/// Schema §2.1.
pub(super) fn read(
    fields: &mut Fields<'_>,
    sections: &mut Sections<'_>,
) -> Result<Organization, Reported> {
    let name = fields.required("name");
    let aliases = fields.list("aliases");
    let kind = fields.required("kind");
    let location = fields.optional("location");
    let website = fields.optional("website");
    let labels = fields.translations("labels");
    let translated_from = fields.translations("translated_from");
    let sources = fields.list("sources");
    let reviewed = fields.required("reviewed");
    let description = sections.paragraphs("Description");
    Ok(Organization {
        name: name?,
        aliases: aliases?,
        kind: kind?,
        location: location?,
        website: website?,
        labels: labels?,
        translated_from: translated_from?,
        sources: sources?,
        reviewed: reviewed?,
        description: description?,
    })
}
