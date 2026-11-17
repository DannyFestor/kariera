use kariera_domain::{
    Contract, Engagement, MonthlyLog, Note, Organization, Profile, Project, Qualification, Skill,
    Source,
};

use crate::context::{Context, Found};

/// A note type with a `sources` list (schema §1.7).
pub(crate) trait Sourced: Note {
    fn sources(&self) -> &[Source];
}

macro_rules! sourced {
    ($($note:ty),+ $(,)?) => {
        $(impl Sourced for $note {
            fn sources(&self) -> &[Source] {
                &self.sources
            }
        })+
    };
}

sourced!(
    Organization,
    Engagement,
    Contract,
    Project,
    MonthlyLog,
    Skill,
    Qualification,
    Profile,
);

/// One Source of one note, with its position in the note's `sources`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SourceOf<'a> {
    pub(crate) file: &'a kariera_vault::NoteFile,
    pub(crate) index: usize,
    pub(crate) source: &'a Source,
}

/// Every Source of every note.
pub(crate) fn all_sources<'a>(context: &Context<'a>) -> Vec<SourceOf<'a>> {
    let mut all = Vec::new();
    collect::<Organization>(context, &mut all);
    collect::<Engagement>(context, &mut all);
    collect::<Contract>(context, &mut all);
    collect::<Project>(context, &mut all);
    collect::<MonthlyLog>(context, &mut all);
    collect::<Skill>(context, &mut all);
    collect::<Qualification>(context, &mut all);
    collect::<Profile>(context, &mut all);
    all
}

fn collect<'a, T: Sourced + 'a>(context: &Context<'a>, all: &mut Vec<SourceOf<'a>>) {
    for Found { file, note, .. } in context.notes::<T>() {
        all.extend(
            note.sources()
                .iter()
                .enumerate()
                .map(|(index, source)| SourceOf {
                    file,
                    index,
                    source,
                }),
        );
    }
}
