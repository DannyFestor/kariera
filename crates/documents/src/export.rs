//! The public career data for the owner's website (issue #36): reviewed notes only, in English, with no personal
//! details, Contracts, Monthly Logs or Sources. Its JSON form is fixed by `schemas/career-export.schema.json`.

use serde::Serialize;
use thiserror::Error;

use kariera_domain::{Career, Engagement, FixedList, Language, Link, Project, TeamSize};

use crate::parts::SpokenLanguage;
use crate::wording::{Named, Wording};

/// Raised whenever the shape of the export changes, so the website can tell which schema a file follows.
pub const EXPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CareerExport {
    pub schema_version: u32,
    pub profile: ExportedProfile,
    pub engagements: Vec<ExportedEngagement>,
    pub projects: Vec<ExportedProject>,
    pub qualifications: Vec<ExportedQualification>,
    pub languages: Vec<SpokenLanguage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportedProfile {
    pub name: String,
    pub headline: String,
    pub summary: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportedEngagement {
    pub note: String,
    pub organization: String,
    pub engagement_type: &'static str,
    pub role: Option<String>,
    pub start: String,
    pub end: Option<String>,
    pub summary: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportedProject {
    pub note: String,
    pub engagement: String,
    pub title: String,
    pub start: String,
    pub end: Option<String>,
    pub role: String,
    pub focus: Option<String>,
    pub team_size: Option<u32>,
    pub industry: String,
    pub skills: Vec<String>,
    pub description: Vec<String>,
    pub achievements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportedQualification {
    pub name: String,
    pub category: &'static str,
    pub issuer: Option<String>,
    pub start: Option<String>,
    pub date: String,
    pub level: Option<String>,
    pub score: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ExportError {
    #[error("the vault has no Profile, which the export starts from")]
    NoProfile,
    #[error("the Profile is not reviewed, and the export holds reviewed notes only")]
    UnreviewedProfile,
}

pub fn export(career: &Career) -> Result<CareerExport, ExportError> {
    let wording = Wording::new(career, Language::English);
    Ok(CareerExport {
        schema_version: EXPORT_SCHEMA_VERSION,
        profile: profile(&wording)?,
        engagements: engagements(&wording),
        projects: projects(&wording),
        qualifications: qualifications(&wording),
        languages: wording
            .human_languages()
            .filter(|(_, skill)| skill.reviewed)
            .map(|language| wording.spoken_language(language))
            .collect(),
    })
}

fn profile(wording: &Wording<'_>) -> Result<ExportedProfile, ExportError> {
    let (name, profile) = wording
        .career
        .profiles
        .iter()
        .next()
        .ok_or(ExportError::NoProfile)?;
    if !profile.reviewed {
        return Err(ExportError::UnreviewedProfile);
    }
    Ok(ExportedProfile {
        name: profile.name.to_string(),
        headline: wording.text(name, &profile.headline),
        summary: wording.paragraphs(name, &profile.summary),
    })
}

fn engagements(wording: &Wording<'_>) -> Vec<ExportedEngagement> {
    wording
        .engagements_newest_first()
        .into_iter()
        .filter(|(_, engagement)| engagement.reviewed)
        .map(|(name, engagement)| ExportedEngagement {
            note: name.to_string(),
            organization: wording.engagement_organization(engagement),
            engagement_type: engagement.engagement_type.slug(),
            role: wording.optional_text(name, engagement.role.as_ref()),
            start: engagement.start.to_string(),
            end: engagement.end.map(|end| end.to_string()),
            summary: wording.paragraphs(name, &engagement.summary),
        })
        .collect()
}

/// A Project of an Engagement the export leaves out is left out too, so every `engagement` names an exported one.
fn projects(wording: &Wording<'_>) -> Vec<ExportedProject> {
    wording
        .projects_newest_first()
        .into_iter()
        .filter(|(_, project)| project.reviewed && is_exported(wording, &project.engagement))
        .map(|named| exported_project(wording, named))
        .collect()
}

fn is_exported(wording: &Wording<'_>, engagement: &Link<Engagement>) -> bool {
    wording
        .career
        .get(engagement)
        .is_some_and(|engagement| engagement.reviewed)
}

fn exported_project(wording: &Wording<'_>, named: Named<'_, Project>) -> ExportedProject {
    let (name, project) = named;
    ExportedProject {
        note: name.to_string(),
        engagement: project.engagement.target().to_string(),
        title: wording.project_title(named),
        start: project.start.to_string(),
        end: project.end.map(|end| end.to_string()),
        role: wording.text(name, &project.role),
        focus: wording.optional_text(name, project.focus.as_ref()),
        team_size: project.team_size.map(TeamSize::get),
        industry: wording.text(name, &project.industry),
        skills: wording.skill_names(project),
        description: wording.paragraphs(name, &project.description),
        achievements: wording.bullets(name, &project.achievements),
    }
}

fn qualifications(wording: &Wording<'_>) -> Vec<ExportedQualification> {
    wording
        .qualifications_newest_first()
        .into_iter()
        .filter(|(_, qualification)| qualification.reviewed)
        .map(|(_, qualification)| ExportedQualification {
            name: qualification.name.to_string(),
            category: qualification.category.slug(),
            issuer: qualification
                .issuer
                .as_ref()
                .map(|issuer| wording.issuer(issuer)),
            start: qualification.start.map(|start| start.to_string()),
            date: qualification.date.to_string(),
            level: qualification.level.as_ref().map(ToString::to_string),
            score: qualification.score.as_ref().map(ToString::to_string),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use kariera_domain::{Career, EngagementType, Issuer, Paragraphs, Translated};

    use super::{CareerExport, ExportError, export};
    use crate::test_career::{
        engagement, english, link, month, organization, profile, project, qualification, skill,
        text,
    };
    use kariera_domain::SkillCategory;

    fn reviewed_career() -> Career {
        let mut career = Career::default();
        let mut owner = profile("Robin Beispiel");
        owner.reviewed = true;
        career.profiles.insert("Profile".parse().unwrap(), owner);
        career
            .organizations
            .insert("Acme".parse().unwrap(), organization("Acme"));
        career
    }

    fn add_engagement(career: &mut Career, name: &str, start: &str, reviewed: bool) {
        let mut added = engagement(EngagementType::Freelance, "Acme", None);
        added.start = month(start);
        added.reviewed = reviewed;
        career.engagements.insert(name.parse().unwrap(), added);
    }

    /// A Project that starts with its Engagement.
    fn add_project(career: &mut Career, name: &str, of: &str, reviewed: bool) {
        let start = career.engagements[&of.parse().unwrap()].start.to_string();
        let mut added = project(&start, None, &["Go"]);
        added.engagement = link(of);
        added.reviewed = reviewed;
        career.projects.insert(name.parse().unwrap(), added);
    }

    fn exported(career: &Career) -> CareerExport {
        export(career).unwrap()
    }

    fn project_notes(export: &CareerExport) -> Vec<&str> {
        export
            .projects
            .iter()
            .map(|project| project.note.as_str())
            .collect()
    }

    #[test]
    fn an_unreviewed_engagement_is_left_out_and_its_projects_with_it() {
        let mut career = reviewed_career();
        add_engagement(&mut career, "2021-04 Acme", "2021-04", false);
        add_engagement(&mut career, "2023-10 Acme", "2023-10", true);
        add_project(&mut career, "Acme - Shop", "2021-04 Acme", true);
        add_project(&mut career, "Acme - Search", "2023-10 Acme", true);

        let export = exported(&career);

        let engagements: Vec<&str> = export
            .engagements
            .iter()
            .map(|engagement| engagement.note.as_str())
            .collect();
        assert_eq!(engagements, ["2023-10 Acme"]);
        assert_eq!(project_notes(&export), ["Acme - Search"]);
    }

    #[test]
    fn an_unreviewed_project_is_left_out() {
        let mut career = reviewed_career();
        add_engagement(&mut career, "2023-10 Acme", "2023-10", true);
        add_project(&mut career, "Acme - Search", "2023-10 Acme", false);

        assert_eq!(project_notes(&exported(&career)), Vec::<&str>::new());
    }

    #[test]
    fn a_confidential_project_exports_its_public_title_and_no_end_client() {
        let mut career = reviewed_career();
        add_engagement(&mut career, "2021-04 Acme", "2021-04", true);
        add_project(&mut career, "Acme - Ledger", "2021-04 Acme", true);
        career
            .organizations
            .insert("Northwind".parse().unwrap(), organization("Northwind"));
        let ledger = career
            .projects
            .get_mut(&"Acme - Ledger".parse().unwrap())
            .unwrap();
        ledger.title = english(text("Northwind freight ledger"));
        ledger.public_title = Some(english(text("Accounting platform")));
        ledger.confidential = true;
        ledger.end_client = Some(link("Northwind"));

        let export = exported(&career);

        assert_eq!(export.projects[0].title, "Accounting platform");
        assert!(!format!("{export:?}").contains("Northwind"), "{export:#?}");
    }

    #[test]
    fn a_public_project_exports_its_title_and_still_no_end_client() {
        let mut career = reviewed_career();
        add_engagement(&mut career, "2021-04 Acme", "2021-04", true);
        add_project(&mut career, "Acme - Shop", "2021-04 Acme", true);
        career
            .organizations
            .insert("Contoso".parse().unwrap(), organization("Contoso"));
        let shop = career
            .projects
            .get_mut(&"Acme - Shop".parse().unwrap())
            .unwrap();
        shop.title = english(text("Toy shop backend"));
        shop.end_client = Some(link("Contoso"));

        let export = exported(&career);

        assert_eq!(export.projects[0].title, "Toy shop backend");
        assert!(!format!("{export:?}").contains("Contoso"), "{export:#?}");
    }

    #[test]
    fn engagements_and_projects_are_newest_first_and_then_by_note_name() {
        let mut career = reviewed_career();
        add_engagement(&mut career, "2019-04 Acme", "2019-04", true);
        add_engagement(&mut career, "2023-10 Acme", "2023-10", true);
        add_project(&mut career, "B", "2019-04 Acme", true);
        add_project(&mut career, "A", "2019-04 Acme", true);
        add_project(&mut career, "C", "2023-10 Acme", true);

        let export = exported(&career);

        assert_eq!(export.engagements[0].note, "2023-10 Acme");
        assert_eq!(project_notes(&export), ["C", "A", "B"]);
    }

    #[test]
    fn an_engagement_reads_its_organization_type_role_period_and_summary_in_english() {
        let mut career = reviewed_career();
        add_engagement(&mut career, "2021-04 Acme", "2021-04", true);
        let acme = career
            .engagements
            .get_mut(&"2021-04 Acme".parse().unwrap())
            .unwrap();
        acme.role = Some(Translated {
            en: text("Backend developer"),
            ja: Some(text("バックエンド開発者")),
            de: Some(text("Backend-Entwickler")),
        });
        acme.summary = english(Paragraphs("Built shops\nin Go.\n\nAnd more.".to_owned()));

        let export = exported(&career);

        let engagement = &export.engagements[0];
        assert_eq!(engagement.organization, "Acme");
        assert_eq!(engagement.engagement_type, "freelance");
        assert_eq!(engagement.role.as_deref(), Some("Backend developer"));
        assert_eq!(engagement.start, "2021-04");
        assert_eq!(engagement.end, None);
        assert_eq!(engagement.summary, ["Built shops in Go.", "And more."]);
    }

    #[test]
    fn a_project_reads_its_fields_in_english_with_its_skill_names() {
        let mut career = reviewed_career();
        add_engagement(&mut career, "2021-04 Acme", "2021-04", true);
        add_project(&mut career, "Acme - Shop", "2021-04 Acme", true);
        career.skills.insert(
            "Go".parse().unwrap(),
            skill("Golang", SkillCategory::Language, None),
        );
        let shop = career
            .projects
            .get_mut(&"Acme - Shop".parse().unwrap())
            .unwrap();
        shop.end = Some(month("2022-09"));
        shop.team_size = Some("5".parse().unwrap());
        shop.focus = Some(english(text("Backend")));
        shop.achievements = english(vec![text("Halved the load time")]);

        let project = &exported(&career).projects[0];

        assert_eq!(project.engagement, "2021-04 Acme");
        assert_eq!(project.end.as_deref(), Some("2022-09"));
        assert_eq!(project.role, "Developer");
        assert_eq!(project.focus.as_deref(), Some("Backend"));
        assert_eq!(project.team_size, Some(5));
        assert_eq!(project.industry, "Retail");
        assert_eq!(project.skills, ["Golang"]);
        assert_eq!(project.achievements, ["Halved the load time"]);
    }

    #[test]
    fn reviewed_qualifications_are_newest_first_with_their_issuer() {
        let mut career = reviewed_career();
        let mut degree = qualification("BSc Computer Science", "2018-03");
        degree.reviewed = true;
        degree.issuer = Some(Issuer::School(link("Acme")));
        degree.start = Some(month("2014-10"));
        let mut certificate = qualification("Cloud Associate", "2024-05");
        certificate.reviewed = true;
        certificate.issuer = Some(Issuer::Body(text("Example Cloud")));
        certificate.score = Some(text("870"));
        let unreviewed = qualification("Driving licence", "2010-01");
        for added in [degree, certificate, unreviewed] {
            career
                .qualifications
                .insert(added.name.as_str().parse().unwrap(), added);
        }

        let export = exported(&career);

        let qualifications: Vec<(&str, &str, Option<&str>)> = export
            .qualifications
            .iter()
            .map(|entry| {
                (
                    entry.name.as_str(),
                    entry.date.as_str(),
                    entry.issuer.as_deref(),
                )
            })
            .collect();
        assert_eq!(
            qualifications,
            [
                ("Cloud Associate", "2024-05", Some("Example Cloud")),
                ("BSc Computer Science", "2018-03", Some("Acme")),
            ]
        );
        assert_eq!(export.qualifications[0].category, "certificate");
        assert_eq!(export.qualifications[0].score.as_deref(), Some("870"));
        assert_eq!(export.qualifications[1].start.as_deref(), Some("2014-10"));
    }

    #[test]
    fn qualifications_of_one_month_are_in_the_order_of_their_names() {
        let mut career = reviewed_career();
        for name in ["B Test", "A Test", "C Test"] {
            let mut added = qualification(name, "2024-05");
            added.reviewed = true;
            career.qualifications.insert(name.parse().unwrap(), added);
        }

        let names: Vec<String> = exported(&career)
            .qualifications
            .into_iter()
            .map(|entry| entry.name)
            .collect();

        assert_eq!(names, ["A Test", "B Test", "C Test"]);
    }

    #[test]
    fn the_languages_are_the_reviewed_human_language_skills() {
        let mut career = reviewed_career();
        let mut japanese = skill("Japanese", SkillCategory::HumanLanguage, None);
        japanese.reviewed = true;
        japanese.level = Some(Translated {
            en: text("Business level"),
            ja: Some(text("ビジネスレベル")),
            de: None,
        });
        let german = skill("German", SkillCategory::HumanLanguage, None);
        let mut go = skill("Go", SkillCategory::Language, None);
        go.reviewed = true;
        for added in [japanese, german, go] {
            career
                .skills
                .insert(added.name.as_str().parse().unwrap(), added);
        }

        let languages = exported(&career).languages;

        assert_eq!(languages.len(), 1);
        assert_eq!(languages[0].name, "Japanese");
        assert_eq!(languages[0].level.as_deref(), Some("Business level"));
    }

    #[test]
    fn the_profile_gives_its_name_headline_and_summary() {
        let mut career = reviewed_career();
        let owner = career
            .profiles
            .get_mut(&"Profile".parse().unwrap())
            .unwrap();
        owner.summary = english(Paragraphs("Full-stack developer.".to_owned()));

        let export = exported(&career);

        assert_eq!(export.schema_version, 1);
        assert_eq!(export.profile.name, "Robin Beispiel");
        assert_eq!(export.profile.headline, "Developer");
        assert_eq!(export.profile.summary, ["Full-stack developer."]);
    }

    #[test]
    fn an_unreviewed_profile_is_refused() {
        let mut career = reviewed_career();
        for owner in career.profiles.values_mut() {
            owner.reviewed = false;
        }

        assert_eq!(export(&career), Err(ExportError::UnreviewedProfile));
    }

    #[test]
    fn a_career_without_a_profile_is_refused() {
        assert_eq!(export(&Career::default()), Err(ExportError::NoProfile));
    }
}
