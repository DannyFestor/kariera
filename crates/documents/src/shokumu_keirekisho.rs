//! The 職務経歴書, Japanese: layout variant B「サマリー先行・カード型」 (issue #7, branch
//! `prototype/shokumu-keirekisho`).

use kariera_domain::{Engagement, Labelled, Project, Qualification, QualificationCategory};
use serde::Serialize;

use crate::context::{Context, Named, url_text};
use crate::labels::ShokumuKeirekishoLabels;
use crate::months::is_within_years;
use crate::parts::{PhaseMark, QualificationEntry, SkillGroup, SpokenLanguage};

const SKILL_WINDOW_YEARS: u32 = 5;
/// A Project that ended within these years, or is still running, gets a full card; older ones one line each.
const DETAIL_YEARS: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShokumuKeirekisho {
    pub labels: ShokumuKeirekishoLabels,
    pub name: String,
    pub headline: String,
    /// 作成日, as the date it states: 「2026年11月18日現在」.
    pub as_of: String,
    pub summary: Vec<String>,
    pub strengths: Vec<String>,
    pub qualifications: Vec<QualificationEntry>,
    pub languages: Vec<SpokenLanguage>,
    /// What the skill table shows, under its heading.
    pub skills_note: String,
    pub skills: Vec<SkillGroup>,
    /// Which Projects get a full card, under the heading of the history.
    pub history_note: String,
    pub engagements: Vec<EngagementHistory>,
    pub self_pr: Vec<String>,
    pub portfolio_url: Option<String>,
    pub github_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EngagementHistory {
    pub organization: String,
    pub period: String,
    pub engagement_type: String,
    pub role: Option<String>,
    /// 事業内容: the Organization's Description, printed only beside full cards.
    pub business: Vec<String>,
    pub cards: Vec<ProjectCard>,
    pub older: Vec<ProjectLine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectCard {
    pub period: String,
    pub title: String,
    pub industry: String,
    pub role: String,
    pub focus: Option<String>,
    pub team: Option<String>,
    pub end_client: Option<String>,
    pub phases: Vec<PhaseMark>,
    pub skills: Vec<String>,
    pub description: Vec<String>,
    pub achievements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectLine {
    pub period: String,
    pub title: String,
    pub industry: String,
    pub skills: Vec<String>,
    pub role: String,
}

pub(crate) fn build(context: &Context<'_>) -> ShokumuKeirekisho {
    let (profile_name, profile) = context.profile;
    ShokumuKeirekisho {
        labels: ShokumuKeirekishoLabels::JAPANESE,
        name: profile.name.to_string(),
        headline: context.text(profile_name, &profile.headline),
        as_of: format!("{}現在", context.locale.date(context.today)),
        summary: context.paragraphs(profile_name, &profile.summary),
        strengths: context.bullets(profile_name, &profile.strengths),
        qualifications: qualifications(context),
        languages: context.spoken_languages(),
        skills_note: format!(
            "直近{SKILL_WINDOW_YEARS}年以内に使用したスキル。期間は案件での使用期間の合計。"
        ),
        skills: context.skill_groups(SKILL_WINDOW_YEARS),
        history_note: format!(
            "直近{DETAIL_YEARS}年以内の案件は詳細を、それ以前の案件は概要を記載。"
        ),
        engagements: context
            .engagements_newest_first()
            .into_iter()
            .map(|engagement| engagement_history(context, engagement))
            .collect(),
        self_pr: context.paragraphs(profile_name, &profile.self_pr),
        portfolio_url: profile.portfolio_url.as_ref().map(url_text),
        github_url: profile.github_url.as_ref().map(url_text),
    }
}

/// Every Qualification but degrees, which belong to the 履歴書, oldest first.
fn qualifications(context: &Context<'_>) -> Vec<QualificationEntry> {
    context
        .qualifications_oldest_first()
        .into_iter()
        .filter(|(_, qualification)| qualification.category != QualificationCategory::Degree)
        .map(|(_, qualification)| qualification_entry(context, qualification))
        .collect()
}

/// The label, the level and the score in brackets.
fn qualification_entry(context: &Context<'_>, qualification: &Qualification) -> QualificationEntry {
    context.qualification_entry(qualification, |label, level, score| {
        let level = level.map_or_else(String::new, |level| format!(" {level}"));
        let score = score.map_or_else(String::new, |score| format!("（{score}）"));
        format!("{label}{level}{score}")
    })
}

fn engagement_history(
    context: &Context<'_>,
    (name, engagement): Named<'_, Engagement>,
) -> EngagementHistory {
    let (recent, older): (Vec<_>, Vec<_>) = context
        .projects_of(name)
        .into_iter()
        .partition(|(_, project)| is_detailed(context, project));
    let business = if recent.is_empty() {
        Vec::new()
    } else {
        business_of(context, engagement)
    };
    EngagementHistory {
        organization: context.engagement_organization(engagement),
        period: context.period(engagement.start, engagement.end),
        engagement_type: engagement
            .engagement_type
            .label(context.language())
            .to_owned(),
        role: context.optional_text(name, engagement.role.as_ref()),
        business,
        cards: recent
            .into_iter()
            .map(|project| card(context, project))
            .collect(),
        older: older
            .into_iter()
            .map(|project| line(context, project))
            .collect(),
    }
}

fn is_detailed(context: &Context<'_>, project: &Project) -> bool {
    project
        .end
        .is_none_or(|end| is_within_years(end, DETAIL_YEARS, context.today.month()))
}

fn business_of(context: &Context<'_>, engagement: &Engagement) -> Vec<String> {
    let link = engagement.client.as_ref().or(engagement.employer.as_ref());
    link.and_then(|link| context.career.organizations.get_key_value(link.target()))
        .map_or_else(Vec::new, |(name, organization)| {
            context.paragraphs(name, &organization.description)
        })
}

fn card(context: &Context<'_>, named: Named<'_, Project>) -> ProjectCard {
    let (name, project) = named;
    ProjectCard {
        period: context.period(project.start, project.end),
        title: context.project_title(named),
        industry: context.text(name, &project.industry),
        role: context.text(name, &project.role),
        focus: context.optional_text(name, project.focus.as_ref()),
        team: project
            .team_size
            .map(|size| context.locale.team(size.get())),
        end_client: context.end_client(project),
        phases: context.phases(project),
        skills: context.skill_names(project),
        description: context.paragraphs(name, &project.description),
        achievements: context.bullets(name, &project.achievements),
    }
}

fn line(context: &Context<'_>, named: Named<'_, Project>) -> ProjectLine {
    let (name, project) = named;
    ProjectLine {
        period: context.period(project.start, project.end),
        title: context.project_title(named),
        industry: context.text(name, &project.industry),
        skills: context.skill_names(project),
        role: context.text(name, &project.role),
    }
}
