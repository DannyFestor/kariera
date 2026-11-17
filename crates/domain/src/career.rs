use std::collections::BTreeMap;

use crate::fixed_list::fixed_list;
use crate::{
    Contract, Engagement, Link, MonthlyLog, NoteName, Organization, Profile, Project,
    Qualification, Skill,
};

fixed_list!(
    /// The `type` field every note starts with (schema §2.0).
    pub NoteType {
        Organization => "organization",
        Engagement => "engagement",
        Contract => "contract",
        Project => "project",
        MonthlyLog => "monthly-log",
        Skill => "skill",
        Qualification => "qualification",
        Profile => "profile",
    }
);

/// The notes of one type, by name.
pub type Notes<T> = BTreeMap<NoteName, T>;

/// Every note of the vault, by type and name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Career {
    pub organizations: Notes<Organization>,
    pub engagements: Notes<Engagement>,
    pub contracts: Notes<Contract>,
    pub projects: Notes<Project>,
    pub monthly_logs: Notes<MonthlyLog>,
    pub skills: Notes<Skill>,
    pub qualifications: Notes<Qualification>,
    /// Exactly one in a valid vault (schema §3), which is a check across notes.
    pub profiles: Notes<Profile>,
}

/// A note type, so a `Link<T>` finds its note among the notes of type `T`.
pub trait Note: Sized {
    const TYPE: NoteType;

    fn all(career: &Career) -> &Notes<Self>;
}

impl Career {
    #[must_use]
    pub fn get<T: Note>(&self, link: &Link<T>) -> Option<&T> {
        T::all(self).get(link.target())
    }
}

macro_rules! note_types {
    ($($note:ident in $field:ident),+ $(,)?) => {
        $(impl Note for $note {
            const TYPE: NoteType = NoteType::$note;

            fn all(career: &Career) -> &Notes<Self> {
                &career.$field
            }
        })+
    };
}

note_types!(
    Organization in organizations,
    Engagement in engagements,
    Contract in contracts,
    Project in projects,
    MonthlyLog in monthly_logs,
    Skill in skills,
    Qualification in qualifications,
    Profile in profiles,
);

#[cfg(test)]
mod tests {
    use super::{Career, Note, NoteType};
    use crate::{FixedList, Link, Month, MonthlyLog, Skill};

    fn log() -> MonthlyLog {
        MonthlyLog {
            engagement: Link::new("2025-06 Acme".parse().unwrap()),
            month: "2025-07".parse::<Month>().unwrap(),
            hours: "152.5".parse().unwrap(),
            project_hours: std::collections::BTreeMap::new(),
            sources: Vec::new(),
            focus: "Shipped the search".parse().unwrap(),
            outcomes: Vec::new(),
        }
    }

    #[test]
    fn a_link_finds_its_note_among_the_notes_of_its_type() {
        let mut career = Career::default();
        career
            .monthly_logs
            .insert("2025-07 Acme Log".parse().unwrap(), log());

        let found = career.get(&Link::<MonthlyLog>::new(
            "2025-07 Acme Log".parse().unwrap(),
        ));

        assert_eq!(found, Some(&log()));
    }

    #[test]
    fn a_link_to_a_missing_note_finds_nothing() {
        let career = Career::default();

        assert_eq!(
            career.get(&Link::<Skill>::new("Rust".parse().unwrap())),
            None
        );
    }

    #[test]
    fn each_note_type_knows_its_type_slug() {
        assert_eq!(MonthlyLog::TYPE.slug(), "monthly-log");
        assert_eq!(NoteType::from_slug("skill"), Some(Skill::TYPE));
    }
}
