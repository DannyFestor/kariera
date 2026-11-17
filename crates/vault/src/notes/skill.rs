use kariera_domain::{FixedList, Skill, SkillCategory};

use crate::fields::{Fields, Presence};
use crate::problems::Reported;
use crate::sections::Sections;

/// Schema §2.6.
pub(super) fn read(
    fields: &mut Fields<'_>,
    _empty_body: &mut Sections<'_>,
) -> Result<Skill, Reported> {
    let name = fields.required("name");
    let aliases = fields.list("aliases");
    let category = fields.required::<SkillCategory>("category");
    let labels = fields.translations("labels");
    let level = fields.conditional("level", category.map(level_presence));
    let translated_from = fields.translations("translated_from");
    let sources = fields.list("sources");
    let reviewed = fields.required("reviewed");
    Ok(Skill {
        name: name?,
        aliases: aliases?,
        category: category?,
        labels: labels?,
        level: level?,
        translated_from: translated_from?,
        sources: sources?,
        reviewed: reviewed?,
    })
}

fn level_presence(category: SkillCategory) -> Presence {
    if category == SkillCategory::HumanLanguage {
        Presence::Optional
    } else {
        Presence::Empty(format!("category is {}", category.slug()))
    }
}
