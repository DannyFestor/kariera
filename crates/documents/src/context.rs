//! What every document reads from the career, in the document's language.

use std::cmp::Reverse;

use kariera_domain::{
    Bullets, Career, Date, Engagement, FixedList, Labelled, Language, Link, Month, NoteName,
    Organization, Paragraphs, Personal, Phase, Profile, Project, Qualification, Skill,
    SkillCategory, Text, Translated, Translations, Url,
};

use crate::experience::Experience;
use crate::locale::Locale;
use crate::parts::{
    PhaseMark, QualificationEntry, SkillEntry, SkillGroup, SpokenLanguage, by_category,
};
use crate::translator::{MissingTranslation, Translator};

/// Everything a document is built from.
#[derive(Debug, Clone, Copy)]
pub struct Inputs<'input> {
    pub career: &'input Career,
    pub personal: &'input Personal,
    /// The build date: the date a document states, and the end of every period still running.
    pub today: Date,
}

/// A note with its name, which a missing translation is reported by.
pub(crate) type Named<'career, T> = (&'career NoteName, &'career T);

#[derive(Debug)]
pub(crate) struct Context<'input> {
    pub(crate) career: &'input Career,
    pub(crate) personal: &'input Personal,
    pub(crate) today: Date,
    pub(crate) locale: Locale,
    pub(crate) profile: Named<'input, Profile>,
    pub(crate) experience: Experience<'input>,
    translator: Translator,
}

impl<'input> Context<'input> {
    /// `None` when the career has no Profile.
    pub(crate) fn new(inputs: &Inputs<'input>, language: Language) -> Option<Self> {
        let profile = inputs.career.profiles.iter().next()?;
        Some(Self {
            career: inputs.career,
            personal: inputs.personal,
            today: inputs.today,
            locale: Locale(language),
            profile,
            experience: Experience::of(inputs.career, inputs.today.month()),
            translator: Translator::new(language),
        })
    }

    pub(crate) const fn language(&self) -> Language {
        self.locale.0
    }

    pub(crate) fn into_missing(self) -> Vec<MissingTranslation> {
        self.translator.into_missing()
    }

    pub(crate) fn text(&self, note: &NoteName, value: &Translated<Text>) -> String {
        self.translator.pick(note, value).to_string()
    }

    pub(crate) fn optional_text(
        &self,
        note: &NoteName,
        value: Option<&Translated<Text>>,
    ) -> Option<String> {
        value.map(|value| self.text(note, value))
    }

    pub(crate) fn paragraphs(
        &self,
        note: &NoteName,
        value: &Translated<Paragraphs>,
    ) -> Vec<String> {
        self.locale.paragraphs(&self.translator.pick(note, value).0)
    }

    pub(crate) fn bullets(&self, note: &NoteName, value: &Translated<Bullets>) -> Vec<String> {
        self.translator
            .pick(note, value)
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    pub(crate) fn label(&self, name: &Text, labels: &Translations<Text>) -> String {
        self.translator.label(name, labels).to_string()
    }

    /// Its `labels` entry for the language, else its `name` (issue #17).
    pub(crate) fn organization(&self, link: &Link<Organization>) -> String {
        self.career.get(link).map_or_else(
            || link.target().to_string(),
            |organization| self.label(&organization.name, &organization.labels),
        )
    }

    /// The Client, or the employer of an employment.
    pub(crate) fn engagement_organization(&self, engagement: &Engagement) -> String {
        engagement
            .client
            .as_ref()
            .or(engagement.employer.as_ref())
            .map_or_else(String::new, |link| self.organization(link))
    }

    /// A confidential Project prints only its public title (schema §1.9).
    pub(crate) fn project_title(&self, (name, project): Named<'_, Project>) -> String {
        let title = project
            .public_title
            .as_ref()
            .filter(|_| project.confidential)
            .unwrap_or(&project.title);
        self.text(name, title)
    }

    /// Never for a confidential Project (schema §1.9).
    pub(crate) fn end_client(&self, project: &Project) -> Option<String> {
        project
            .end_client
            .as_ref()
            .filter(|_| !project.confidential)
            .map(|link| self.organization(link))
    }

    pub(crate) fn skill_name(&self, link: &Link<Skill>) -> String {
        self.career.get(link).map_or_else(
            || link.target().to_string(),
            |skill| self.label(&skill.name, &skill.labels),
        )
    }

    pub(crate) fn skill_names(&self, project: &Project) -> Vec<String> {
        project
            .skills
            .iter()
            .map(|link| self.skill_name(link))
            .collect()
    }

    /// The Skills used within `years`, by category (issue #7).
    pub(crate) fn skill_groups(&self, years: u32) -> Vec<SkillGroup> {
        let entries = self.experience.recent(years).into_iter().map(|used| {
            let entry = SkillEntry {
                name: self.label(&used.skill.name, &used.skill.labels),
                months: used.months,
                experience: self.locale.experience(used.months),
                last_use: self.locale.month(used.last_use),
            };
            (used.skill.category, entry)
        });
        by_category(entries)
            .into_iter()
            .map(|(category, skills)| SkillGroup {
                category: category.label(self.language()).to_owned(),
                skills,
            })
            .collect()
    }

    pub(crate) fn spoken_languages(&self) -> Vec<SpokenLanguage> {
        self.career
            .skills
            .iter()
            .filter(|(_, skill)| skill.category == SkillCategory::HumanLanguage)
            .map(|(name, skill)| SpokenLanguage {
                name: self.label(&skill.name, &skill.labels),
                level: self.optional_text(name, skill.level.as_ref()),
            })
            .collect()
    }

    pub(crate) fn phases(&self, project: &Project) -> Vec<PhaseMark> {
        Phase::ALL
            .iter()
            .map(|&phase| PhaseMark {
                label: phase.label(self.language()).to_owned(),
                done: project.phases.contains(&phase),
            })
            .collect()
    }

    /// `days_per_week_min` to `days_per_week_max`.
    pub(crate) fn days_per_week(&self) -> Option<String> {
        let (_, profile) = self.profile;
        let min = profile.days_per_week_min?;
        let max = profile.days_per_week_max.unwrap_or(min);
        Some(self.locale.range(min.get(), max.get()))
    }

    pub(crate) fn period(&self, start: Month, end: Option<Month>) -> String {
        self.locale.period(start, end)
    }

    pub(crate) fn engagements_newest_first(&self) -> Vec<Named<'input, Engagement>> {
        let mut engagements: Vec<Named<'input, Engagement>> =
            self.career.engagements.iter().collect();
        engagements.sort_by_key(|(_, engagement)| Reverse(engagement.start));
        engagements
    }

    pub(crate) fn projects_newest_first(&self) -> Vec<Named<'input, Project>> {
        let mut projects: Vec<Named<'input, Project>> = self.career.projects.iter().collect();
        projects.sort_by_key(|(_, project)| Reverse(project.start));
        projects
    }

    pub(crate) fn projects_of(&self, engagement: &NoteName) -> Vec<Named<'input, Project>> {
        self.projects_newest_first()
            .into_iter()
            .filter(|(_, project)| project.engagement.target() == engagement)
            .collect()
    }

    /// The entry of a Qualification whose title `compose` writes from its label, its `level` unless the label
    /// already says it, and its `score`.
    pub(crate) fn qualification_entry(
        &self,
        qualification: &Qualification,
        compose: impl FnOnce(String, Option<String>, Option<String>) -> String,
    ) -> QualificationEntry {
        let label = self.label(&qualification.name, &qualification.labels);
        let level = qualification
            .level
            .as_ref()
            .map(ToString::to_string)
            .filter(|level| !label.contains(level.as_str()));
        let score = qualification.score.as_ref().map(ToString::to_string);
        QualificationEntry {
            title: compose(label, level, score),
            issuer: qualification.issuer.as_ref().map(ToString::to_string),
            date: self.locale.month(qualification.date),
        }
    }

    pub(crate) fn qualifications_oldest_first(&self) -> Vec<Named<'input, Qualification>> {
        let mut qualifications: Vec<Named<'input, Qualification>> =
            self.career.qualifications.iter().collect();
        qualifications.sort_by_key(|(_, qualification)| qualification.date);
        qualifications
    }
}

pub(crate) fn url_text(url: &Url) -> String {
    url.as_str().to_owned()
}
