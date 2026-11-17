use std::io;
use std::path::PathBuf;

use kariera_domain::{FixedList, Month, NoteName, ValueError};
use thiserror::Error;

use crate::creation::request::OrganizationRole;

#[derive(Debug, Error)]
pub enum CreateError {
    #[error("no Engagement is named [[{0}]]")]
    UnknownEngagement(NoteName),
    #[error("[[{0}]] links neither a client nor an employer")]
    EngagementWithoutOrganization(NoteName),
    #[error("no Organization is named [[{0}]]")]
    UnknownOrganization(NoteName),
    #[error("[[{engagement}]] runs from {start} to {end}, so it has no log for {month}", end = end_text(*.end))]
    MonthOutsideEngagement {
        engagement: NoteName,
        month: Month,
        start: Month,
        end: Option<Month>,
    },
    #[error("an Engagement of type {} has a {} and no {}", .engagement_type.slug(), .expected.field(), .given.field())]
    RoleMismatch {
        engagement_type: kariera_domain::EngagementType,
        expected: OrganizationRole,
        given: OrganizationRole,
    },
    #[error("[[{organization}]] is not of kind {}, which its role in the Engagement needs", .needed.join(" or "))]
    WrongOrganizationKind {
        organization: NoteName,
        needed: Vec<&'static str>,
    },
    #[error("{name:?} is not a valid note name: {source}")]
    InvalidName { name: String, source: ValueError },
    #[error("{} already has that name, ignoring case", path.display())]
    NameTaken { path: PathBuf },
    #[error("{} has no empty `{field}` field to fill", template.display())]
    TemplateField { template: PathBuf, field: String },
    #[error("cannot read {}: {source}", path.display())]
    Unreadable { path: PathBuf, source: io::Error },
    #[error("cannot write {}: {source}", path.display())]
    Unwritable { path: PathBuf, source: io::Error },
}

fn end_text(end: Option<Month>) -> String {
    end.map_or_else(|| "now".to_owned(), |end| end.to_string())
}
