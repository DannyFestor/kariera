//! One parser per note type. Each reads the fields of its schema table (§2) and the body sections the schema
//! lists for it; a type without sections reads none, which asks for an empty body.

mod contract;
mod engagement;
mod monthly_log;
mod organization;
mod profile;
mod project;
mod qualification;
mod skill;

use kariera_domain::{Career, NoteName, NoteType, Notes};

use crate::Problem;
use crate::body::Body;
use crate::cursor::Cursor;
use crate::fields::Fields;
use crate::names::Names;
use crate::problems::{Problems, Reported};
use crate::sections::Sections;
use crate::yaml::Node;

type Parser<T> = fn(&mut Fields<'_>, &mut Sections<'_>) -> Result<T, Reported>;

/// What a parser reads: one note's frontmatter and body, and the vault's names for its links.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NoteSource<'a> {
    pub(crate) frontmatter: &'a Node,
    pub(crate) body: &'a Body,
    pub(crate) names: &'a Names,
}

/// Adds the note to `career` as the type its `type` field gives.
pub(crate) fn add(
    career: &mut Career,
    name: NoteName,
    note_type: NoteType,
    source: NoteSource<'_>,
) -> Result<(), Vec<Problem>> {
    match note_type {
        NoteType::Organization => {
            insert(&mut career.organizations, name, source, organization::read)
        }
        NoteType::Engagement => insert(&mut career.engagements, name, source, engagement::read),
        NoteType::Contract => insert(&mut career.contracts, name, source, contract::read),
        NoteType::Project => insert(&mut career.projects, name, source, project::read),
        NoteType::MonthlyLog => insert(&mut career.monthly_logs, name, source, monthly_log::read),
        NoteType::Skill => insert(&mut career.skills, name, source, skill::read),
        NoteType::Qualification => insert(
            &mut career.qualifications,
            name,
            source,
            qualification::read,
        ),
        NoteType::Profile => insert(&mut career.profiles, name, source, profile::read),
    }
}

fn insert<T>(
    notes: &mut Notes<T>,
    name: NoteName,
    source: NoteSource<'_>,
    parse: Parser<T>,
) -> Result<(), Vec<Problem>> {
    notes.insert(name, read(source, parse)?);
    Ok(())
}

fn read<T>(source: NoteSource<'_>, parse: Parser<T>) -> Result<T, Vec<Problem>> {
    let problems = Problems::default();
    let cursor = Cursor::root(source.names);
    let value = Fields::new(source.frontmatter, cursor, &problems).and_then(|mut fields| {
        fields.skip("type");
        let mut sections = Sections::new(source.body, &problems);
        let value = parse(&mut fields, &mut sections);
        fields.finish();
        sections.finish();
        value
    });
    problems.into_result(value)
}
