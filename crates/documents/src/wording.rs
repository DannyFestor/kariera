//! The career's notes as one language writes them: no personal details and no build date, so the public export
//! words the notes exactly as the documents do.

use std::cmp::Reverse;

use kariera_domain::{
    Bullets, Career, Engagement, FixedList, Issuer, Labelled, Language, Link, Month, NoteName,
    Organization, Paragraphs, Phase, Project, Qualification, Skill, SkillCategory, Text,
    Translated, Translations,
};

use crate::locale::Locale;
use crate::parts::{PhaseMark, QualificationEntry, SpokenLanguage};
use crate::translator::{MissingTranslation, Translator};

/// A note with its name, which a missing translation is reported by.
pub(crate) type Named<'career, T> = (&'career NoteName, &'career T);

#[derive(Debug)]
pub(crate) struct Wording<'career> {
    pub(crate) career: &'career Career,
    pub(crate) locale: Locale,
    translator: Translator,
}

impl<'career> Wording<'career> {
    pub(crate) const fn new(career: &'career Career, language: Language) -> Self {
        Self {
            career,
            locale: Locale(language),
            translator: Translator::new(language),
        }
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

    /// The counterparties of the Engagement's Contracts other than its Client, which are Agencies (schema §2.3), each
    /// once.
    pub(crate) fn agencies(&self, engagement: &Engagement) -> Vec<String> {
        let mut agencies: Vec<String> = Vec::new();
        let counterparties = engagement
            .contracts
            .iter()
            .filter_map(|link| self.career.get(link))
            .map(|contract| &contract.counterparty)
            .filter(|&counterparty| engagement.client.as_ref() != Some(counterparty));
        for counterparty in counterparties {
            let agency = self.organization(counterparty);
            if !agencies.contains(&agency) {
                agencies.push(agency);
            }
        }
        agencies
    }

    /// A school by its label, like any Organization; an issuing body as it is written.
    pub(crate) fn issuer(&self, issuer: &Issuer) -> String {
        match issuer {
            Issuer::School(link) => self.organization(link),
            Issuer::Body(body) => body.to_string(),
        }
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

    pub(crate) fn spoken_languages(&self) -> Vec<SpokenLanguage> {
        self.human_languages()
            .map(|language| self.spoken_language(language))
            .collect()
    }

    pub(crate) fn human_languages(
        &self,
    ) -> impl Iterator<Item = Named<'career, Skill>> + use<'career> {
        self.career
            .skills
            .iter()
            .filter(|(_, skill)| skill.category == SkillCategory::HumanLanguage)
    }

    pub(crate) fn spoken_language(&self, (name, skill): Named<'_, Skill>) -> SpokenLanguage {
        SpokenLanguage {
            name: self.label(&skill.name, &skill.labels),
            level: self.optional_text(name, skill.level.as_ref()),
        }
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

    pub(crate) fn period(&self, start: Month, end: Option<Month>) -> String {
        self.locale.period(start, end)
    }

    pub(crate) fn engagements_newest_first(&self) -> Vec<Named<'career, Engagement>> {
        let mut engagements: Vec<Named<'career, Engagement>> =
            self.career.engagements.iter().collect();
        engagements.sort_by_key(|(_, engagement)| Reverse(engagement.start));
        engagements
    }

    pub(crate) fn projects_newest_first(&self) -> Vec<Named<'career, Project>> {
        let mut projects: Vec<Named<'career, Project>> = self.career.projects.iter().collect();
        projects.sort_by_key(|(_, project)| Reverse(project.start));
        projects
    }

    pub(crate) fn projects_of(&self, engagement: &NoteName) -> Vec<Named<'career, Project>> {
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
            issuer: qualification
                .issuer
                .as_ref()
                .map(|issuer| self.issuer(issuer)),
            date: self.locale.month(qualification.date),
        }
    }

    pub(crate) fn qualifications_oldest_first(&self) -> Vec<Named<'career, Qualification>> {
        let mut qualifications: Vec<Named<'career, Qualification>> =
            self.career.qualifications.iter().collect();
        qualifications.sort_by_key(|(_, qualification)| qualification.date);
        qualifications
    }

    pub(crate) fn qualifications_newest_first(&self) -> Vec<Named<'career, Qualification>> {
        let mut qualifications: Vec<Named<'career, Qualification>> =
            self.career.qualifications.iter().collect();
        qualifications.sort_by_key(|(_, qualification)| Reverse(qualification.date));
        qualifications
    }
}
