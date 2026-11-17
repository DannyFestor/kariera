//! One module per concern of the schema. Each check reads the whole loaded vault and returns what it found.

mod aliases;
mod body;
mod confidentiality;
mod contract_rules;
mod duplicate_names;
mod file_names;
mod forbidden_content;
mod frontmatter;
mod hours;
mod link_roles;
mod periods;
mod placement;
mod profile;
mod rates;
mod skill_dates;
mod sources;

use crate::context::Context;
use crate::finding::Finding;

type Check = fn(&Context<'_>) -> Vec<Finding>;

const CHECKS: [Check; 16] = [
    placement::check,
    file_names::check,
    duplicate_names::check,
    frontmatter::check,
    periods::check,
    skill_dates::check,
    link_roles::check,
    contract_rules::check,
    rates::check,
    hours::check,
    aliases::check,
    sources::check,
    body::check,
    confidentiality::check,
    forbidden_content::check,
    profile::check,
];

pub(crate) fn run(context: &Context<'_>) -> Vec<Finding> {
    CHECKS.iter().flat_map(|check| check(context)).collect()
}
