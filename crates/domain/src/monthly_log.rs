use std::collections::BTreeMap;

use crate::{Bullets, Engagement, Hours, Link, Month, Project, Source, Text};

/// Schema §2.5. English only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthlyLog {
    pub engagement: Link<Engagement>,
    pub month: Month,
    pub hours: Hours,
    pub project_hours: BTreeMap<Link<Project>, Hours>,
    pub sources: Vec<Source>,
    pub focus: Text,
    pub outcomes: Bullets,
}
