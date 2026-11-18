//! The Skill Sheet, Japanese: layout C, one block per Project (issue #9, branch `prototype/skill-sheet`).

use kariera_domain::{
    Contract, Labelled, Link, Project, QualificationCategory, SkillCategory, TeamSize,
};
use serde::Serialize;

use crate::context::{Context, Named, url_text};
use crate::months::MonthSpan;
use crate::parts::{PhaseMark, QualificationEntry, SkillGroup, by_category};

const SKILL_WINDOW_YEARS: u32 = 5;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillSheet {
    pub name: String,
    /// 更新日: the build month.
    pub updated: String,
    pub headline: String,
    /// 所属: the owner's business.
    pub business: String,
    pub available_from: Option<String>,
    pub days_per_week: Option<String>,
    pub portfolio_url: Option<String>,
    pub github_url: Option<String>,
    pub certificates: Vec<QualificationEntry>,
    pub education: Vec<QualificationEntry>,
    pub strengths: Vec<String>,
    pub skill_window_years: u32,
    pub skills: Vec<SkillGroup>,
    /// The Summary followed by the Self-PR.
    pub self_pr: Vec<String>,
    pub projects: Vec<SheetProject>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SheetProject {
    pub number: usize,
    pub title: String,
    pub period: String,
    pub duration: String,
    pub client: String,
    pub end_client: Option<String>,
    pub industry: String,
    /// The types of the Engagement's Contracts: a Project does not link its own.
    pub contract_type: String,
    pub role: String,
    pub focus: Option<String>,
    pub team_size: Option<u32>,
    pub phases: Vec<PhaseMark>,
    pub technology: Vec<Technology>,
    pub description: Vec<String>,
    pub achievements: Vec<String>,
}

/// The Skills of a Project in one category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Technology {
    pub category: String,
    pub skills: Vec<String>,
}

pub(crate) fn build(context: &Context<'_>) -> SkillSheet {
    let (profile_name, profile) = context.profile;
    let mut self_pr = context.paragraphs(profile_name, &profile.summary);
    self_pr.extend(context.paragraphs(profile_name, &profile.self_pr));
    SkillSheet {
        name: profile.name.to_string(),
        updated: context.locale.month(context.today.month()),
        headline: context.text(profile_name, &profile.headline),
        business: context.organization(&profile.business),
        available_from: profile.available_from.map(|date| context.locale.date(date)),
        days_per_week: context.days_per_week(),
        portfolio_url: profile.portfolio_url.as_ref().map(url_text),
        github_url: profile.github_url.as_ref().map(url_text),
        certificates: qualifications(context, |category| {
            matches!(
                category,
                QualificationCategory::Certificate | QualificationCategory::TestResult
            )
        }),
        education: qualifications(context, |category| {
            category == QualificationCategory::Degree
        }),
        strengths: context.bullets(profile_name, &profile.strengths),
        skill_window_years: SKILL_WINDOW_YEARS,
        skills: context.skill_groups(SKILL_WINDOW_YEARS),
        self_pr,
        projects: context
            .projects_newest_first()
            .into_iter()
            .enumerate()
            .map(|(index, project)| sheet_project(context, index + 1, project))
            .collect(),
    }
}

fn qualifications(
    context: &Context<'_>,
    is_listed: impl Fn(QualificationCategory) -> bool,
) -> Vec<QualificationEntry> {
    context
        .qualifications_oldest_first()
        .into_iter()
        .filter(|(_, qualification)| is_listed(qualification.category))
        .map(|(_, qualification)| context.qualification_entry(qualification, |label, _, _| label))
        .collect()
}

fn sheet_project(context: &Context<'_>, number: usize, named: Named<'_, Project>) -> SheetProject {
    let (name, project) = named;
    let engagement = context.career.get(&project.engagement);
    let months = MonthSpan::of(project.start, project.end, context.today.month()).len();
    SheetProject {
        number,
        title: context.project_title(named),
        period: context.period(project.start, project.end),
        duration: context.locale.experience(months),
        client: engagement.map_or_else(String::new, |engagement| {
            context.engagement_organization(engagement)
        }),
        end_client: context.end_client(project),
        industry: context.text(name, &project.industry),
        contract_type: engagement.map_or_else(String::new, |engagement| {
            contract_types(context, &engagement.contracts)
        }),
        role: context.text(name, &project.role),
        focus: context.optional_text(name, project.focus.as_ref()),
        team_size: project.team_size.map(TeamSize::get),
        phases: context.phases(project),
        technology: technology(context, project),
        description: context.paragraphs(name, &project.description),
        achievements: context.bullets(name, &project.achievements),
    }
}

fn contract_types(context: &Context<'_>, contracts: &[Link<Contract>]) -> String {
    let mut labels: Vec<&str> = Vec::new();
    for contract in contracts.iter().filter_map(|link| context.career.get(link)) {
        let label = contract.contract_type.label(context.language());
        if !labels.contains(&label) {
            labels.push(label);
        }
    }
    labels.join("・")
}

/// The Project's Skills by category, in the schema's order of categories.
fn technology(context: &Context<'_>, project: &Project) -> Vec<Technology> {
    let mut skills: Vec<(SkillCategory, String)> = project
        .skills
        .iter()
        .filter_map(|link| {
            let skill = context.career.get(link)?;
            Some((skill.category, context.skill_name(link)))
        })
        .collect();
    skills.sort_by_key(|(category, _)| *category);
    by_category(skills)
        .into_iter()
        .map(|(category, skills)| Technology {
            category: category.label(context.language()).to_owned(),
            skills,
        })
        .collect()
}
