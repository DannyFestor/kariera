//! PROTOTYPE, throwaway (branch `prototype/cv`, issues #7 and #8): writes the CV content of a vault as one JSON file
//! per language, for the layout variants in `crates/render-typst/prototype/cv/`. The real content model belongs in
//! the `documents` crate; this only exists so the variants can be judged on real content.
//!
//! Lives in `cli` because only `cli` may read a vault and build documents from it (ADR 0001).
//!
//! `cargo run -p kariera-cli --example prototype_cv_data -- <vault> <out-dir> <build month YYYY-MM>`

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use kariera_domain::{
    Career, Engagement, FixedList, Labelled, Language, Month, Paragraphs, Profile, Project,
    Qualification, Skill, SkillCategory, Text, Translated, TranslationLanguage,
};
use kariera_vault::Vault;

const LANGUAGES: [Language; 2] = [Language::English, Language::German];

fn main() -> Result<(), String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [vault_root, out_dir, build_month] = arguments.as_slice() else {
        return Err("usage: prototype_cv_data <vault> <out-dir> <build month YYYY-MM>".to_owned());
    };
    let build_month = build_month
        .parse::<Month>()
        .map_err(|error| error.to_string())?;
    let career = load_career(Path::new(vault_root))?;
    fs::create_dir_all(out_dir).map_err(|error| error.to_string())?;
    for language in LANGUAGES {
        let cv = CvWriter {
            career: &career,
            language,
            build_month,
        }
        .cv()?;
        let path = PathBuf::from(out_dir).join(format!("cv-{}.json", language.code()));
        fs::write(&path, cv.render()).map_err(|error| error.to_string())?;
    }
    Ok(())
}

// Lenient on purpose: a note that does not read yet should not keep the owner from judging the layout.
fn load_career(root: &Path) -> Result<Career, String> {
    let inspection = Vault::inspect(root).map_err(|error| error.to_string())?;
    Ok(inspection.vault.career().clone())
}

struct CvWriter<'career> {
    career: &'career Career,
    language: Language,
    build_month: Month,
}

impl CvWriter<'_> {
    fn cv(&self) -> Result<Json, String> {
        let profile = self
            .career
            .profiles
            .values()
            .next()
            .ok_or("the vault has no Profile")?;
        Ok(Json::object([
            ("lang", Json::string(self.language.code())),
            ("build_month", Json::string(self.build_month.to_string())),
            ("profile", self.profile(profile)),
            ("engagements", self.engagements()),
            ("skills", self.skills()),
            ("languages", self.human_languages()),
            ("qualifications", self.qualifications()),
        ]))
    }

    fn profile(&self, profile: &Profile) -> Json {
        Json::object([
            ("name", Json::string(profile.name.as_str())),
            ("headline", self.text(&profile.headline)),
            (
                "business",
                self.organization_name(profile.business.target().as_str()),
            ),
            ("summary", self.paragraphs(&profile.summary)),
            ("strengths", self.bullets(&profile.strengths)),
            (
                "available_from",
                Json::optional(profile.available_from.map(|date| date.to_string())),
            ),
            (
                "days_per_week_min",
                Json::optional(profile.days_per_week_min.map(|days| days.get().to_string())),
            ),
            (
                "days_per_week_max",
                Json::optional(profile.days_per_week_max.map(|days| days.get().to_string())),
            ),
            (
                "portfolio_url",
                Json::optional(
                    profile
                        .portfolio_url
                        .as_ref()
                        .map(|url| url.as_str().to_owned()),
                ),
            ),
            (
                "github_url",
                Json::optional(
                    profile
                        .github_url
                        .as_ref()
                        .map(|url| url.as_str().to_owned()),
                ),
            ),
        ])
    }

    fn engagements(&self) -> Json {
        let mut engagements: Vec<(&str, &Engagement)> = self
            .career
            .engagements
            .iter()
            .map(|(name, engagement)| (name.as_str(), engagement))
            .collect();
        engagements.sort_by_key(|(_, engagement)| std::cmp::Reverse(engagement.start));
        Json::Array(
            engagements
                .into_iter()
                .map(|(name, engagement)| self.engagement(name, engagement))
                .collect(),
        )
    }

    fn engagement(&self, name: &str, engagement: &Engagement) -> Json {
        let organization = engagement
            .client
            .as_ref()
            .or(engagement.employer.as_ref())
            .map_or_default(|link| link.target().as_str().to_owned());
        Json::object([
            ("organization", self.organization_name(&organization)),
            ("type", Json::string(engagement.engagement_type.slug())),
            (
                "type_label",
                Json::string(engagement.engagement_type.label(self.language)),
            ),
            ("role", self.optional_text(engagement.role.as_ref())),
            ("start", Json::string(engagement.start.to_string())),
            (
                "end",
                Json::optional(engagement.end.map(|end| end.to_string())),
            ),
            ("summary", self.paragraphs(&engagement.summary)),
            ("projects", self.projects_of(name)),
        ])
    }

    fn projects_of(&self, engagement_name: &str) -> Json {
        let mut projects: Vec<&Project> = self
            .career
            .projects
            .values()
            .filter(|project| project.engagement.target().as_str() == engagement_name)
            .collect();
        projects.sort_by_key(|project| std::cmp::Reverse(project.start));
        Json::Array(
            projects
                .into_iter()
                .map(|project| self.project(project))
                .collect(),
        )
    }

    fn project(&self, project: &Project) -> Json {
        Json::object([
            ("title", self.text(shown_title(project))),
            ("start", Json::string(project.start.to_string())),
            (
                "end",
                Json::optional(project.end.map(|end| end.to_string())),
            ),
            ("role", self.text(&project.role)),
            ("focus", self.optional_text(project.focus.as_ref())),
            ("industry", self.text(&project.industry)),
            (
                "team_size",
                Json::optional(project.team_size.map(|size| size.get().to_string())),
            ),
            ("end_client", self.shown_end_client(project)),
            ("phases", self.phases(project)),
            ("skills", self.project_skills(project)),
            ("description", self.paragraphs(&project.description)),
            ("achievements", self.bullets(&project.achievements)),
        ])
    }

    // A confidential Project never names its End Client (schema §1.9).
    fn shown_end_client(&self, project: &Project) -> Json {
        project
            .end_client
            .as_ref()
            .filter(|_| !project.confidential)
            .map_or(Json::Null, |link| {
                self.organization_name(link.target().as_str())
            })
    }

    fn phases(&self, project: &Project) -> Json {
        Json::Array(
            project
                .phases
                .iter()
                .map(|phase| Json::string(phase.label(self.language)))
                .collect(),
        )
    }

    fn project_skills(&self, project: &Project) -> Json {
        Json::Array(
            project
                .skills
                .iter()
                .map(|link| Json::string(self.skill_label(link.target().as_str())))
                .collect(),
        )
    }

    fn skills(&self) -> Json {
        let months_by_skill = self.months_by_skill();
        Json::Array(
            self.career
                .skills
                .iter()
                .filter(|(_, skill)| skill.category != SkillCategory::HumanLanguage)
                .map(|(name, skill)| {
                    let months = months_by_skill.get(name.as_str());
                    self.skill(skill, months)
                })
                .collect(),
        )
    }

    fn skill(&self, skill: &Skill, months: Option<&BTreeSet<u32>>) -> Json {
        let month_count = months.map_or(0, BTreeSet::len);
        let last_used = months
            .and_then(|months| months.last().copied())
            .map(month_from_index);
        Json::object([
            ("name", Json::string(self.skill_label(skill.name.as_str()))),
            ("category", Json::string(skill.category.slug())),
            (
                "category_label",
                Json::string(skill.category.label(self.language)),
            ),
            ("months", Json::string(month_count.to_string())),
            ("last_used", Json::optional(last_used)),
        ])
    }

    // Years of experience per Skill: the union of the month ranges of its Projects (schema §4).
    fn months_by_skill(&self) -> BTreeMap<&str, BTreeSet<u32>> {
        let mut months_by_skill: BTreeMap<&str, BTreeSet<u32>> = BTreeMap::new();
        for project in self.career.projects.values() {
            let first = month_index(project.start);
            let last = month_index(project.end.unwrap_or(self.build_month));
            for link in &project.skills {
                months_by_skill
                    .entry(link.target().as_str())
                    .or_default()
                    .extend(first..=last);
            }
        }
        months_by_skill
    }

    fn human_languages(&self) -> Json {
        Json::Array(
            self.career
                .skills
                .values()
                .filter(|skill| skill.category == SkillCategory::HumanLanguage)
                .map(|skill| {
                    Json::object([
                        ("name", Json::string(self.skill_label(skill.name.as_str()))),
                        ("level", self.optional_text(skill.level.as_ref())),
                    ])
                })
                .collect(),
        )
    }

    fn qualifications(&self) -> Json {
        let mut qualifications: Vec<&Qualification> = self.career.qualifications.values().collect();
        qualifications.sort_by_key(|qualification| std::cmp::Reverse(qualification.date));
        Json::Array(
            qualifications
                .into_iter()
                .map(|qualification| self.qualification(qualification))
                .collect(),
        )
    }

    fn qualification(&self, qualification: &Qualification) -> Json {
        let name = self
            .translation_language()
            .and_then(|language| qualification.labels.get(language))
            .unwrap_or(&qualification.name);
        Json::object([
            ("name", Json::string(name.as_str())),
            ("category", Json::string(qualification.category.slug())),
            ("issuer", optional_plain(qualification.issuer.as_ref())),
            (
                "start",
                Json::optional(qualification.start.map(|start| start.to_string())),
            ),
            ("date", Json::string(qualification.date.to_string())),
            ("level", optional_plain(qualification.level.as_ref())),
            ("score", optional_plain(qualification.score.as_ref())),
        ])
    }

    fn organization_name(&self, note_name: &str) -> Json {
        let name = self
            .career
            .organizations
            .iter()
            .find(|(name, _)| name.as_str() == note_name)
            .map_or(note_name, |(_, organization)| organization.name.as_str());
        Json::string(name)
    }

    fn skill_label(&self, note_name: &str) -> String {
        let skill = self
            .career
            .skills
            .iter()
            .find(|(name, _)| name.as_str() == note_name)
            .map(|(_, skill)| skill);
        let label = skill.and_then(|skill| {
            self.translation_language()
                .and_then(|language| skill.labels.get(language))
        });
        label.map_or_else(
            || {
                skill
                    .map_or(note_name, |skill| skill.name.as_str())
                    .to_owned()
            },
            ToString::to_string,
        )
    }

    // An untranslated value falls back to English: an open question for the owner, not a decision.
    fn text(&self, value: &Translated<Text>) -> Json {
        let text = value.get(self.language).unwrap_or(&value.en);
        Json::string(text.as_str())
    }

    fn optional_text(&self, value: Option<&Translated<Text>>) -> Json {
        value.map_or(Json::Null, |value| self.text(value))
    }

    fn paragraphs(&self, value: &Translated<Paragraphs>) -> Json {
        let paragraphs = value
            .get(self.language)
            .filter(|paragraphs| !paragraphs.0.is_empty())
            .unwrap_or(&value.en);
        Json::Array(
            paragraphs
                .0
                .split("\n\n")
                .filter(|paragraph| !paragraph.trim().is_empty())
                .map(|paragraph| Json::string(paragraph.replace('\n', " ")))
                .collect(),
        )
    }

    fn bullets(&self, value: &Translated<Vec<Text>>) -> Json {
        let bullets = value
            .get(self.language)
            .filter(|bullets| !bullets.is_empty())
            .unwrap_or(&value.en);
        Json::Array(
            bullets
                .iter()
                .map(|bullet| Json::string(bullet.as_str()))
                .collect(),
        )
    }

    const fn translation_language(&self) -> Option<TranslationLanguage> {
        match self.language {
            Language::English => None,
            Language::Japanese => Some(TranslationLanguage::Japanese),
            Language::German => Some(TranslationLanguage::German),
        }
    }
}

fn shown_title(project: &Project) -> &Translated<Text> {
    project
        .public_title
        .as_ref()
        .filter(|_| project.confidential)
        .unwrap_or(&project.title)
}

fn optional_plain(text: Option<&Text>) -> Json {
    text.map_or(Json::Null, |text| Json::string(text.as_str()))
}

fn month_index(month: Month) -> u32 {
    let written = month.to_string();
    let (year, month_of_year) = written.split_once('-').unwrap_or(("0", "1"));
    let year: u32 = year.parse().unwrap_or(0);
    let month_of_year: u32 = month_of_year.parse().unwrap_or(1);
    year * 12 + month_of_year - 1
}

fn month_from_index(index: u32) -> String {
    format!("{:04}-{:02}", index / 12, index % 12 + 1)
}

enum Json {
    Null,
    String(String),
    Array(Vec<Self>),
    Object(Vec<(&'static str, Self)>),
}

impl Json {
    fn string(text: impl Into<String>) -> Self {
        Self::String(text.into())
    }

    fn optional(text: Option<String>) -> Self {
        text.map_or(Self::Null, Self::String)
    }

    fn object<const N: usize>(members: [(&'static str, Self); N]) -> Self {
        Self::Object(members.into())
    }

    fn render(&self) -> String {
        match self {
            Self::Null => "null".to_owned(),
            Self::String(text) => quoted(text),
            Self::Array(items) => {
                let items: Vec<String> = items.iter().map(Self::render).collect();
                format!("[{}]", items.join(","))
            }
            Self::Object(members) => {
                let members: Vec<String> = members
                    .iter()
                    .map(|(key, value)| format!("{}:{}", quoted(key), value.render()))
                    .collect();
                format!("{{{}}}", members.join(","))
            }
        }
    }
}

fn quoted(text: &str) -> String {
    let mut quoted = String::from("\"");
    for character in text.chars() {
        match character {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\t' => quoted.push_str("\\t"),
            other => quoted.push(other),
        }
    }
    quoted.push('"');
    quoted
}
