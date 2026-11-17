//! PROTOTYPE, throwaway: feeds the 職務経歴書 / 履歴書 layout prototypes in
//! `crates/render-typst/prototype/shokumu-keirekisho/`. The real content model belongs in the `documents` crate
//! (issue #7); this only dumps the raw notes the templates need as JSON.
//!
//! `cargo run -p kariera-cli --example prototype_shokumu_keirekisho_data -- <vault> <out.json>`

use std::error::Error;
use std::fmt::Write as _;
use std::path::PathBuf;

use kariera_domain::{
    Career, Engagement, Labelled, Language, Link, Month, Organization, Paragraphs, Project,
    Qualification, Skill, Text, Translated,
};
use kariera_vault::Vault;

enum Json {
    Null,
    Number(u32),
    String(String),
    Array(Vec<Self>),
    Object(Vec<(&'static str, Self)>),
}

impl Json {
    fn text(text: &impl ToString) -> Self {
        Self::String(text.to_string())
    }

    fn optional<T>(value: Option<T>, to_json: impl FnOnce(T) -> Self) -> Self {
        value.map_or(Self::Null, to_json)
    }

    fn write(&self, out: &mut String) {
        match self {
            Self::Null => out.push_str("null"),
            Self::Number(number) => out.push_str(&number.to_string()),
            Self::String(string) => write_string(string, out),
            Self::Array(items) => {
                out.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    item.write(out);
                }
                out.push(']');
            }
            Self::Object(fields) => {
                out.push('{');
                for (index, (key, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    write_string(key, out);
                    out.push(':');
                    value.write(out);
                }
                out.push('}');
            }
        }
    }
}

fn write_string(string: &str, out: &mut String) {
    out.push('"');
    for character in string.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            control if control.is_control() => {
                let _ignored = write!(out, "\\u{:04x}", u32::from(control));
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

fn translated<T>(value: &Translated<T>, to_json: impl Fn(&T) -> Json) -> Json {
    Json::Object(vec![
        ("en", to_json(&value.en)),
        ("ja", Json::optional(value.ja.as_ref(), &to_json)),
    ])
}

fn translated_text(value: &Translated<Text>) -> Json {
    translated(value, Json::text)
}

fn paragraphs(value: &Translated<Paragraphs>) -> Json {
    translated(value, |paragraphs| {
        if paragraphs.0.is_empty() {
            Json::Null
        } else {
            Json::text(&paragraphs.0)
        }
    })
}

fn bullets(value: &Translated<Vec<Text>>) -> Json {
    translated(value, |bullets| {
        if bullets.is_empty() {
            Json::Null
        } else {
            Json::Array(bullets.iter().map(Json::text).collect())
        }
    })
}

fn month(value: Option<Month>) -> Json {
    Json::optional(value, |month| Json::text(&month))
}

fn link<T>(value: Option<&Link<T>>) -> Json {
    Json::optional(value, |link| Json::text(link.target()))
}

fn label<T: Labelled>(value: T) -> Json {
    Json::Object(vec![
        ("slug", Json::text(&value.slug())),
        ("ja", Json::text(&value.label(Language::Japanese))),
    ])
}

fn organization(organization: &Organization) -> Json {
    Json::Object(vec![
        ("name", Json::text(&organization.name)),
        (
            "aliases",
            Json::Array(organization.aliases.iter().map(Json::text).collect()),
        ),
        (
            "kind",
            Json::Array(organization.kind.iter().copied().map(label).collect()),
        ),
        ("description", paragraphs(&organization.description)),
    ])
}

fn agencies(career: &Career, engagement: &Engagement, organization: Option<&str>) -> Json {
    let counterparties = engagement
        .contracts
        .iter()
        .filter_map(|contract| career.get(contract))
        .map(|contract| contract.counterparty.target().to_string())
        .filter(|counterparty| Some(counterparty.as_str()) != organization);
    let mut names: Vec<String> = counterparties.collect();
    names.dedup();
    Json::Array(names.iter().map(Json::text).collect())
}

fn engagement(career: &Career, name: &str, engagement: &Engagement) -> Json {
    let organization = engagement.client.as_ref().or(engagement.employer.as_ref());
    let organization_name = organization.map(|link| link.target().as_str());
    Json::Object(vec![
        ("name", Json::text(&name)),
        ("organization", link(organization)),
        ("type", label(engagement.engagement_type)),
        (
            "role",
            Json::optional(engagement.role.as_ref(), translated_text),
        ),
        ("start", month(Some(engagement.start))),
        ("end", month(engagement.end)),
        ("summary", paragraphs(&engagement.summary)),
        ("agencies", agencies(career, engagement, organization_name)),
    ])
}

fn shown_title(project: &Project) -> &Translated<Text> {
    project
        .public_title
        .as_ref()
        .filter(|_| project.confidential)
        .unwrap_or(&project.title)
}

fn project_skills(project: &Project) -> Json {
    Json::Array(
        project
            .skills
            .iter()
            .map(|skill| Json::text(skill.target()))
            .collect(),
    )
}

fn project(project: &Project) -> Json {
    let end_client = project
        .end_client
        .as_ref()
        .filter(|_| !project.confidential);
    Json::Object(vec![
        ("engagement", Json::text(project.engagement.target())),
        ("end_client", link(end_client)),
        ("title", translated_text(shown_title(project))),
        ("start", month(Some(project.start))),
        ("end", month(project.end)),
        ("role", translated_text(&project.role)),
        (
            "focus",
            Json::optional(project.focus.as_ref(), translated_text),
        ),
        (
            "team_size",
            Json::optional(project.team_size, |size| Json::Number(size.get())),
        ),
        ("industry", translated_text(&project.industry)),
        (
            "phases",
            Json::Array(project.phases.iter().copied().map(label).collect()),
        ),
        ("skills", project_skills(project)),
        ("description", paragraphs(&project.description)),
        ("achievements", bullets(&project.achievements)),
    ])
}

fn skill(skill: &Skill) -> Json {
    Json::Object(vec![
        ("name", Json::text(&skill.name)),
        ("ja", Json::optional(skill.labels.ja.as_ref(), Json::text)),
        ("category", label(skill.category)),
        (
            "level",
            Json::optional(skill.level.as_ref(), translated_text),
        ),
    ])
}

fn qualification(qualification: &Qualification) -> Json {
    let optional_text = |value: Option<&Text>| Json::optional(value, Json::text);
    Json::Object(vec![
        ("name", Json::text(&qualification.name)),
        ("ja", optional_text(qualification.labels.ja.as_ref())),
        ("category", label(qualification.category)),
        ("issuer", optional_text(qualification.issuer.as_ref())),
        ("start", month(qualification.start)),
        ("date", month(Some(qualification.date))),
        ("level", optional_text(qualification.level.as_ref())),
        ("score", optional_text(qualification.score.as_ref())),
    ])
}

fn profile(career: &Career) -> Json {
    Json::optional(career.profiles.values().next(), |profile| {
        Json::Object(vec![
            ("name", Json::text(&profile.name)),
            ("headline", translated_text(&profile.headline)),
            ("summary", paragraphs(&profile.summary)),
            ("strengths", bullets(&profile.strengths)),
            ("self_pr", paragraphs(&profile.self_pr)),
            (
                "available_from",
                Json::optional(profile.available_from, |date| Json::text(&date)),
            ),
            (
                "days_per_week",
                Json::Array(
                    [profile.days_per_week_min, profile.days_per_week_max]
                        .into_iter()
                        .map(|days| Json::optional(days, |days| Json::Number(days.get())))
                        .collect(),
                ),
            ),
            (
                "portfolio_url",
                Json::optional(profile.portfolio_url.as_ref(), |url| {
                    Json::text(&url.as_str())
                }),
            ),
            (
                "github_url",
                Json::optional(profile.github_url.as_ref(), |url| Json::text(&url.as_str())),
            ),
        ])
    })
}

fn career_json(career: &Career) -> Json {
    Json::Object(vec![
        ("profile", profile(career)),
        (
            "organizations",
            Json::Array(career.organizations.values().map(organization).collect()),
        ),
        (
            "engagements",
            Json::Array(
                career
                    .engagements
                    .iter()
                    .map(|(name, value)| engagement(career, name.as_str(), value))
                    .collect(),
            ),
        ),
        (
            "projects",
            Json::Array(career.projects.values().map(project).collect()),
        ),
        (
            "skills",
            Json::Array(career.skills.values().map(skill).collect()),
        ),
        (
            "qualifications",
            Json::Array(career.qualifications.values().map(qualification).collect()),
        ),
    ])
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args_os().skip(1).map(PathBuf::from);
    let (Some(vault_root), Some(output)) = (arguments.next(), arguments.next()) else {
        return Err("usage: prototype_shokumu_keirekisho_data <vault> <out.json>".into());
    };
    let vault = Vault::load(&vault_root)?;
    let mut json = String::new();
    career_json(vault.career()).write(&mut json);
    std::fs::write(output, json)?;
    Ok(())
}
