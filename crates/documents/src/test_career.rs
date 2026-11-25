//! Small domain values for the unit tests, with every field a test does not care about left empty.

use kariera_domain::{
    Career, Engagement, EngagementType, Link, Month, Organization, OrganizationKind, Paragraphs,
    Profile, Project, Qualification, QualificationCategory, Skill, SkillCategory, Text, Translated,
    Translations,
};

pub(crate) fn month(text: &str) -> Month {
    text.parse().unwrap()
}

pub(crate) fn text(value: &str) -> Text {
    value.parse().unwrap()
}

pub(crate) fn english<T>(value: T) -> Translated<T> {
    Translated {
        en: value,
        ja: None,
        de: None,
    }
}

pub(crate) fn project(start: &str, end: Option<&str>, skills: &[&str]) -> Project {
    Project {
        engagement: Link::new("2021-04 Acme".parse().unwrap()),
        end_client: None,
        title: english(text("Shop")),
        public_title: None,
        confidential: false,
        aliases: Vec::new(),
        start: month(start),
        end: end.map(month),
        role: english(text("Developer")),
        focus: None,
        team_size: None,
        industry: english(text("Retail")),
        phases: Vec::new(),
        skills: skills
            .iter()
            .map(|skill| Link::new(skill.parse().unwrap()))
            .collect(),
        translated_from: Translations::none(),
        sources: Vec::new(),
        reviewed: false,
        description: english(Paragraphs(String::new())),
        achievements: english(Vec::new()),
    }
}

pub(crate) fn engagement(
    engagement_type: EngagementType,
    client: &str,
    end: Option<&str>,
) -> Engagement {
    Engagement {
        client: Some(Link::new(client.parse().unwrap())),
        employer: None,
        engagement_type,
        role: None,
        start: month("2020-01"),
        end: end.map(month),
        contracts: Vec::new(),
        translated_from: Translations::none(),
        sources: Vec::new(),
        reviewed: false,
        summary: english(Paragraphs(String::new())),
    }
}

pub(crate) fn skill(name: &str, category: SkillCategory, last_used: Option<&str>) -> Skill {
    Skill {
        name: text(name),
        aliases: Vec::new(),
        category,
        labels: Translations::none(),
        level: None,
        last_used: last_used.map(month),
        translated_from: Translations::none(),
        sources: Vec::new(),
        reviewed: false,
    }
}

pub(crate) fn career(projects: Vec<Project>, skills: Vec<Skill>) -> Career {
    let mut career = Career::default();
    for (number, project) in projects.into_iter().enumerate() {
        let name = format!("Project {number}").parse().unwrap();
        career.projects.insert(name, project);
    }
    for skill in skills {
        career
            .skills
            .insert(skill.name.as_str().parse().unwrap(), skill);
    }
    career
}

pub(crate) fn link<T>(name: &str) -> Link<T> {
    Link::new(name.parse().unwrap())
}

pub(crate) fn organization(name: &str) -> Organization {
    Organization {
        name: text(name),
        aliases: Vec::new(),
        kind: vec![OrganizationKind::Client],
        location: None,
        website: None,
        labels: Translations::none(),
        translated_from: Translations::none(),
        sources: Vec::new(),
        reviewed: false,
        description: english(Paragraphs(String::new())),
    }
}

pub(crate) fn profile(name: &str) -> Profile {
    Profile {
        name: text(name),
        headline: english(text("Developer")),
        business: link("Example Works"),
        available_from: None,
        days_per_week_min: None,
        days_per_week_max: None,
        portfolio_url: None,
        github_url: None,
        translated_from: Translations::none(),
        sources: Vec::new(),
        reviewed: false,
        summary: english(Paragraphs(String::new())),
        strengths: english(Vec::new()),
        self_pr: english(Paragraphs(String::new())),
    }
}

pub(crate) fn qualification(name: &str, date: &str) -> Qualification {
    Qualification {
        name: text(name),
        labels: Translations::none(),
        category: QualificationCategory::Certificate,
        issuer: None,
        start: None,
        date: month(date),
        level: None,
        score: None,
        translated_from: Translations::none(),
        sources: Vec::new(),
        reviewed: false,
    }
}
