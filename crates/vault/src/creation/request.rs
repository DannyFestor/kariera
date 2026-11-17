use kariera_domain::{Career, EngagementType, Month, NoteName, Text};

use crate::creation::error::CreateError;
use crate::creation::plan::Plan;

/// What the author asks for: the note type with the arguments that name the note and fill its fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewNote {
    Log(NewLog),
    Project(NewProject),
    Engagement(NewEngagement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewLog {
    pub engagement: NoteName,
    pub month: Month,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewProject {
    pub engagement: NoteName,
    /// The short English name of schema §1.1; it also starts `title.en`.
    pub title: Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEngagement {
    pub organization: NoteName,
    pub role: OrganizationRole,
    pub engagement_type: EngagementType,
    pub start: Month,
}

/// The field of an Engagement that links its Organization (schema §2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrganizationRole {
    Client,
    Employer,
}

impl OrganizationRole {
    #[must_use]
    pub const fn field(self) -> &'static str {
        match self {
            Self::Client => "client",
            Self::Employer => "employer",
        }
    }

    /// Schema §2.2: freelance and own-business work has a client, work as an employee an employer.
    #[must_use]
    pub const fn required_by(engagement_type: EngagementType) -> Self {
        match engagement_type {
            EngagementType::Freelance | EngagementType::OwnBusiness => Self::Client,
            EngagementType::Employment | EngagementType::Dispatch | EngagementType::Internship => {
                Self::Employer
            }
        }
    }
}

impl NewNote {
    pub(crate) fn plan(&self, career: &Career) -> Result<Plan, CreateError> {
        match self {
            Self::Log(log) => Plan::for_log(log, career),
            Self::Project(project) => Plan::for_project(project, career),
            Self::Engagement(engagement) => Plan::for_engagement(engagement, career),
        }
    }
}
