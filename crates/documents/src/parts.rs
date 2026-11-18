//! Model parts more than one document prints.

use kariera_domain::SkillCategory;
use serde::Serialize;

/// The Skills of one category in a skill table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillGroup {
    pub category: String,
    pub skills: Vec<SkillEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillEntry {
    pub name: String,
    /// Computed from the Project periods (schema §4), for a bar or a sort.
    pub months: u32,
    /// `months` as the language writes a length of time.
    pub experience: String,
    pub last_use: String,
}

/// A human-language Skill, printed outside the skill tables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpokenLanguage {
    pub name: String,
    pub level: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QualificationEntry {
    pub title: String,
    pub issuer: Option<String>,
    pub date: String,
}

/// One of the six phases (schema §2.4), with whether the Project covered it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PhaseMark {
    pub label: String,
    pub done: bool,
}

/// Items already ordered by category, gathered into one list per category.
pub(crate) fn by_category<T>(
    items: impl IntoIterator<Item = (SkillCategory, T)>,
) -> Vec<(SkillCategory, Vec<T>)> {
    let mut groups: Vec<(SkillCategory, Vec<T>)> = Vec::new();
    for (category, item) in items {
        match groups.last_mut() {
            Some((last, group)) if *last == category => group.push(item),
            Some(_) | None => groups.push((category, vec![item])),
        }
    }
    groups
}
