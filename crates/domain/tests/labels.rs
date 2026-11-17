//! The labels kariera renders must equal the tables of schema §1.12, so they are snapshotted as those tables.

use kariera_domain::{
    ContractType, EngagementType, Labelled, Language, OrganizationKind, Phase,
    QualificationCategory, RateUnit, SkillCategory,
};

fn table<T: Labelled>(title: &str) -> String {
    let rows: Vec<String> = T::ALL
        .iter()
        .map(|value| {
            let labels = Language::ALL.map(|language| value.label(language));
            format!("| `{}` | {} |", value.slug(), labels.join(" | "))
        })
        .collect();
    format!(
        "**{title}**\n\n| Slug | en | ja | de |\n|---|---|---|---|\n{}\n",
        rows.join("\n")
    )
}

#[test]
fn labels_equal_the_schema_tables() {
    let tables = [
        table::<Phase>("Phase"),
        table::<EngagementType>("Engagement type"),
        table::<ContractType>("Contract type"),
        table::<RateUnit>("Rate unit"),
        table::<OrganizationKind>("Organization kind"),
        table::<SkillCategory>("Skill category"),
        table::<QualificationCategory>("Qualification category"),
    ];

    insta::assert_snapshot!(tables.join("\n"));
}
