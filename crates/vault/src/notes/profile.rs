use kariera_domain::Profile;

use crate::fields::Fields;
use crate::problems::Reported;
use crate::sections::Sections;

/// Schema §2.8.
pub(super) fn read(
    fields: &mut Fields<'_>,
    sections: &mut Sections<'_>,
) -> Result<Profile, Reported> {
    let name = fields.required("name");
    let headline = fields.required("headline");
    let business = fields.required("business");
    let available_from = fields.optional("available_from");
    let days_per_week_min = fields.optional("days_per_week_min");
    let days_per_week_max = fields.optional("days_per_week_max");
    let portfolio_url = fields.optional("portfolio_url");
    let github_url = fields.optional("github_url");
    let translated_from = fields.translations("translated_from");
    let sources = fields.list("sources");
    let reviewed = fields.required("reviewed");
    let summary = sections.paragraphs("Summary");
    let strengths = sections.bullets("Strengths");
    let self_pr = sections.paragraphs("Self-PR");
    Ok(Profile {
        name: name?,
        headline: headline?,
        business: business?,
        available_from: available_from?,
        days_per_week_min: days_per_week_min?,
        days_per_week_max: days_per_week_max?,
        portfolio_url: portfolio_url?,
        github_url: github_url?,
        translated_from: translated_from?,
        sources: sources?,
        reviewed: reviewed?,
        summary: summary?,
        strengths: strengths?,
        self_pr: self_pr?,
    })
}
