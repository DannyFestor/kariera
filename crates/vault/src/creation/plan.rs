use std::path::PathBuf;

use kariera_domain::{
    Career, Engagement, EngagementType, FixedList, Link, Month, NoteName, NoteType,
    OrganizationKind,
};

use crate::creation::error::CreateError;
use crate::creation::fill::Fill;
use crate::creation::request::{NewEngagement, NewLog, NewProject, OrganizationRole};
use crate::discovery::{monthly_log_folder, note_folder};

/// Where a note goes and what is written into its template, decided from the vault before anything is written.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Plan {
    pub(crate) note_type: NoteType,
    folder: PathBuf,
    pub(crate) name: NoteName,
    pub(crate) fills: Vec<Fill>,
}

impl Plan {
    /// Relative to the vault root.
    pub(crate) fn path(&self) -> PathBuf {
        self.folder.join(format!("{}.md", self.name))
    }

    pub(crate) fn for_log(log: &NewLog, career: &Career) -> Result<Self, CreateError> {
        let engagement = engagement(career, &log.engagement)?;
        if !is_within(log.month, engagement) {
            return Err(CreateError::MonthOutsideEngagement {
                engagement: log.engagement.clone(),
                month: log.month,
                start: engagement.start,
                end: engagement.end,
            });
        }
        let name = note_name(format!(
            "{} {} Log",
            log.month,
            party(engagement, &log.engagement)?
        ))?;
        Ok(Self {
            note_type: NoteType::MonthlyLog,
            folder: monthly_log_folder(log.month),
            name,
            fills: vec![
                Fill::link("engagement", &log.engagement),
                Fill::plain("month", log.month),
            ],
        })
    }

    pub(crate) fn for_project(project: &NewProject, career: &Career) -> Result<Self, CreateError> {
        let engagement = engagement(career, &project.engagement)?;
        let name = note_name(format!(
            "{} - {}",
            party(engagement, &project.engagement)?,
            project.title
        ))?;
        Ok(Self {
            note_type: NoteType::Project,
            folder: PathBuf::from(note_folder(NoteType::Project)),
            name,
            fills: vec![
                Fill::link("engagement", &project.engagement),
                Fill::text("title.en", &project.title),
            ],
        })
    }

    pub(crate) fn for_engagement(
        engagement: &NewEngagement,
        career: &Career,
    ) -> Result<Self, CreateError> {
        check_role(engagement)?;
        check_kind(engagement, career)?;
        let name = note_name(format!("{} {}", engagement.start, engagement.organization))?;
        Ok(Self {
            note_type: NoteType::Engagement,
            folder: PathBuf::from(note_folder(NoteType::Engagement)),
            name,
            fills: vec![
                Fill::link(engagement.role.field(), &engagement.organization),
                Fill::plain("engagement_type", engagement.engagement_type.slug()),
                Fill::plain("start", engagement.start),
            ],
        })
    }
}

fn engagement<'a>(career: &'a Career, name: &NoteName) -> Result<&'a Engagement, CreateError> {
    career
        .engagements
        .get(name)
        .ok_or_else(|| CreateError::UnknownEngagement(name.clone()))
}

/// The `<Client>` of the file name patterns (schema §1.1).
fn party<'a>(engagement: &'a Engagement, name: &NoteName) -> Result<&'a NoteName, CreateError> {
    engagement
        .client
        .as_ref()
        .or(engagement.employer.as_ref())
        .map(Link::target)
        .ok_or_else(|| CreateError::EngagementWithoutOrganization(name.clone()))
}

fn is_within(month: Month, engagement: &Engagement) -> bool {
    month >= engagement.start && engagement.end.is_none_or(|end| month <= end)
}

fn note_name(name: String) -> Result<NoteName, CreateError> {
    name.parse()
        .map_err(|source| CreateError::InvalidName { name, source })
}

fn check_role(engagement: &NewEngagement) -> Result<(), CreateError> {
    let expected = OrganizationRole::required_by(engagement.engagement_type);
    if engagement.role == expected {
        Ok(())
    } else {
        Err(CreateError::RoleMismatch {
            engagement_type: engagement.engagement_type,
            expected,
            given: engagement.role,
        })
    }
}

/// Schema §3: a client is of kind `client` (`my-business` for own business), an employer of kind `employer`.
fn check_kind(engagement: &NewEngagement, career: &Career) -> Result<(), CreateError> {
    let organization = career
        .organizations
        .get(&engagement.organization)
        .ok_or_else(|| CreateError::UnknownOrganization(engagement.organization.clone()))?;
    let needed = needed_kinds(engagement);
    if organization.kind.iter().any(|kind| needed.contains(kind)) {
        Ok(())
    } else {
        Err(CreateError::WrongOrganizationKind {
            organization: engagement.organization.clone(),
            needed: needed.iter().map(|kind| kind.slug()).collect(),
        })
    }
}

fn needed_kinds(engagement: &NewEngagement) -> Vec<OrganizationKind> {
    match (engagement.role, engagement.engagement_type) {
        (OrganizationRole::Employer, _) => vec![OrganizationKind::Employer],
        (OrganizationRole::Client, EngagementType::OwnBusiness) => {
            vec![OrganizationKind::MyBusiness]
        }
        (OrganizationRole::Client, _) => vec![OrganizationKind::Client],
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use kariera_domain::Career;

    use super::Plan;
    use crate::Vault;
    use crate::creation::error::CreateError;
    use crate::creation::request::{NewLog, NewProject};

    fn career_without_organization_links() -> Career {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/vault");
        let mut career = Vault::load(&root).unwrap().career().clone();
        for engagement in career.engagements.values_mut() {
            engagement.client = None;
            engagement.employer = None;
        }
        career
    }

    #[test]
    fn an_engagement_that_links_no_organization_has_no_name_to_use() {
        let career = career_without_organization_links();
        let engagement = "2023-10 Globex".parse().unwrap();

        let log = Plan::for_log(
            &NewLog {
                engagement: "2023-10 Globex".parse().unwrap(),
                month: "2026-11".parse().unwrap(),
            },
            &career,
        );
        let project = Plan::for_project(
            &NewProject {
                engagement: "2023-10 Globex".parse().unwrap(),
                title: "Billing".parse().unwrap(),
            },
            &career,
        );

        for error in [log.unwrap_err(), project.unwrap_err()] {
            assert!(
                matches!(&error, CreateError::EngagementWithoutOrganization(name) if *name == engagement),
                "{error}"
            );
        }
    }
}
