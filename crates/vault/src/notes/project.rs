use kariera_domain::Project;

use crate::fields::{Fields, Presence};
use crate::problems::Reported;
use crate::sections::Sections;

/// Schema §2.4 and §1.9.
pub(super) fn read(
    fields: &mut Fields<'_>,
    sections: &mut Sections<'_>,
) -> Result<Project, Reported> {
    let engagement = fields.required("engagement");
    let end_client = fields.optional("end_client");
    let title = fields.required("title");
    let confidential = fields.required::<bool>("confidential");
    let public_title = fields.conditional("public_title", confidential.map(public_title_presence));
    let aliases = fields.list("aliases");
    let start = fields.required("start");
    let end = fields.optional("end");
    let role = fields.required("role");
    let focus = fields.optional("focus");
    let team_size = fields.optional("team_size");
    let industry = fields.required("industry");
    let phases = fields.list("phases");
    let skills = fields.required("skills");
    let translated_from = fields.translations("translated_from");
    let sources = fields.required("sources");
    let reviewed = fields.required("reviewed");
    let description = sections.paragraphs("Description");
    let achievements = sections.bullets("Achievements");
    Ok(Project {
        engagement: engagement?,
        end_client: end_client?,
        title: title?,
        public_title: public_title?,
        confidential: confidential?,
        aliases: aliases?,
        start: start?,
        end: end?,
        role: role?,
        focus: focus?,
        team_size: team_size?,
        industry: industry?,
        phases: phases?,
        skills: skills?,
        translated_from: translated_from?,
        sources: sources?,
        reviewed: reviewed?,
        description: description?,
        achievements: achievements?,
    })
}

fn public_title_presence(confidential: bool) -> Presence {
    if confidential {
        Presence::Required("confidential is true".to_owned())
    } else {
        Presence::Empty("confidential is false".to_owned())
    }
}
