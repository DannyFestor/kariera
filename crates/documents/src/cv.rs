//! The CV, English and German: layout variant C, "Projects first" (issue #7, branch `prototype/cv`).

use std::cmp::Reverse;

use kariera_domain::{
    Engagement, Labelled, Language, Personal, Photo, Project, Qualification, QualificationCategory,
    TeamSize,
};
use serde::Serialize;

use crate::context::{Context, Named, url_text};
use crate::parts::{QualificationEntry, SkillGroup, SpokenLanguage};

const SELECTED_PROJECTS: usize = 5;
const ACHIEVEMENTS_PER_PROJECT: usize = 3;
const SKILL_WINDOW_YEARS: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Cv {
    pub language: &'static str,
    pub name: String,
    pub headline: String,
    pub contact: Contact,
    /// Only on the German CV, as German readers expect it.
    #[serde(skip)]
    pub photo: Option<Photo>,
    pub availability: Option<Availability>,
    pub languages: Vec<SpokenLanguage>,
    /// Only on the German CV.
    pub personal_details: Option<PersonalDetails>,
    pub summary: Vec<String>,
    pub strengths: Vec<String>,
    pub projects: Vec<CvProject>,
    pub career: Vec<CareerEntry>,
    pub skills: Vec<SkillGroup>,
    pub education: Vec<QualificationEntry>,
    pub certificates: Vec<QualificationEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Contact {
    pub location: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub portfolio_url: Option<String>,
    pub github_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Availability {
    pub from: String,
    pub days_per_week: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PersonalDetails {
    pub birth_date: Option<String>,
    pub nationality: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CvProject {
    pub title: String,
    pub period: String,
    pub role: String,
    pub focus: Option<String>,
    pub client: String,
    pub end_client: Option<String>,
    pub industry: String,
    pub team_size: Option<u32>,
    /// The first paragraph of the Description.
    pub description: Option<String>,
    pub achievements: Vec<String>,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CareerEntry {
    pub period: String,
    /// The Engagement's role, else its type.
    pub role: String,
    pub organization: String,
    pub engagement_type: String,
    pub projects: usize,
}

pub(crate) fn build(context: &Context<'_>) -> Cv {
    let (profile_name, profile) = context.profile;
    let is_german = context.language() == Language::German;
    let (education, certificates) = qualifications(context);
    Cv {
        language: context.language().code(),
        name: profile.name.to_string(),
        headline: context.text(profile_name, &profile.headline),
        contact: contact(context),
        photo: context.personal.photo.clone().filter(|_| is_german),
        availability: availability(context),
        languages: context.spoken_languages(),
        personal_details: personal_details(context, context.personal).filter(|_| is_german),
        summary: context.paragraphs(profile_name, &profile.summary),
        strengths: context.bullets(profile_name, &profile.strengths),
        projects: selected_projects(context),
        career: career(context),
        skills: context.skill_groups(SKILL_WINDOW_YEARS),
        education,
        certificates,
    }
}

fn contact(context: &Context<'_>) -> Contact {
    let (_, profile) = context.profile;
    let personal = context.personal;
    Contact {
        location: personal.location.as_ref().map(ToString::to_string),
        phone: personal.phone.as_ref().map(ToString::to_string),
        email: personal.email.as_ref().map(ToString::to_string),
        portfolio_url: profile.portfolio_url.as_ref().map(url_text),
        github_url: profile.github_url.as_ref().map(url_text),
    }
}

fn availability(context: &Context<'_>) -> Option<Availability> {
    let (_, profile) = context.profile;
    let from = profile.available_from?;
    Some(Availability {
        from: context.locale.date(from),
        days_per_week: context.days_per_week(),
    })
}

fn personal_details(context: &Context<'_>, personal: &Personal) -> Option<PersonalDetails> {
    let details = PersonalDetails {
        birth_date: personal.birth_date.map(|date| context.locale.date(date)),
        nationality: personal.nationality.as_ref().map(ToString::to_string),
    };
    (details.birth_date.is_some() || details.nationality.is_some()).then_some(details)
}

/// The most recent Projects of all Engagements: running ones first, then by their end.
fn selected_projects(context: &Context<'_>) -> Vec<CvProject> {
    let mut projects = context.projects_newest_first();
    projects.sort_by_key(|(_, project)| {
        Reverse((project.end.is_none(), project.end.unwrap_or(project.start)))
    });
    projects
        .into_iter()
        .take(SELECTED_PROJECTS)
        .map(|project| cv_project(context, project))
        .collect()
}

fn cv_project(context: &Context<'_>, named: Named<'_, Project>) -> CvProject {
    let (name, project) = named;
    let mut achievements = context.bullets(name, &project.achievements);
    achievements.truncate(ACHIEVEMENTS_PER_PROJECT);
    CvProject {
        title: context.project_title(named),
        period: context.period(project.start, project.end),
        role: context.text(name, &project.role),
        focus: context.optional_text(name, project.focus.as_ref()),
        client: client_of(context, project),
        end_client: context.end_client(project),
        industry: context.text(name, &project.industry),
        team_size: project.team_size.map(TeamSize::get),
        description: context
            .paragraphs(name, &project.description)
            .into_iter()
            .next(),
        achievements,
        skills: context.skill_names(project),
    }
}

fn client_of(context: &Context<'_>, project: &Project) -> String {
    context
        .career
        .get(&project.engagement)
        .map_or_else(String::new, |engagement| {
            context.engagement_organization(engagement)
        })
}

fn career(context: &Context<'_>) -> Vec<CareerEntry> {
    context
        .engagements_newest_first()
        .into_iter()
        .map(|engagement| career_entry(context, engagement))
        .collect()
}

fn career_entry(context: &Context<'_>, (name, engagement): Named<'_, Engagement>) -> CareerEntry {
    let engagement_type = engagement
        .engagement_type
        .label(context.language())
        .to_owned();
    CareerEntry {
        period: context.period(engagement.start, engagement.end),
        role: context
            .optional_text(name, engagement.role.as_ref())
            .unwrap_or_else(|| engagement_type.clone()),
        organization: context.engagement_organization(engagement),
        engagement_type,
        projects: context.projects_of(name).len(),
    }
}

/// Degrees, then every other Qualification, each newest first.
fn qualifications(context: &Context<'_>) -> (Vec<QualificationEntry>, Vec<QualificationEntry>) {
    let mut qualifications = context.qualifications_oldest_first();
    qualifications.reverse();
    let (degrees, others): (Vec<_>, Vec<_>) = qualifications
        .into_iter()
        .partition(|(_, qualification)| qualification.category == QualificationCategory::Degree);
    let entries = |list: Vec<Named<'_, Qualification>>| {
        list.into_iter()
            .map(|(_, qualification)| qualification_entry(context, qualification))
            .collect()
    };
    (entries(degrees), entries(others))
}

/// A degree's `level` names the degree its label already names, so only other Qualifications add it.
fn qualification_entry(context: &Context<'_>, qualification: &Qualification) -> QualificationEntry {
    let is_degree = qualification.category == QualificationCategory::Degree;
    context.qualification_entry(qualification, |label, level, score| {
        let details = level.into_iter().chain(score).filter(|_| !is_degree);
        std::iter::once(label)
            .chain(details)
            .collect::<Vec<_>>()
            .join(", ")
    })
}
