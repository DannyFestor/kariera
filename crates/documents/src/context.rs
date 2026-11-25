//! What every document reads from the career, in the document's language.

use std::borrow::Cow;
use std::ops::Deref;

use kariera_domain::{Career, Date, Labelled, Personal, Profile, Url};

use crate::edition::Edition;
use crate::experience::Experience;
use crate::parts::{SkillEntry, SkillGroup, by_category};
use crate::translator::MissingTranslation;
use crate::wording::Wording;

pub(crate) use crate::wording::Named;

/// Everything a document is built from.
#[derive(Debug, Clone, Copy)]
pub struct Inputs<'input> {
    pub career: &'input Career,
    pub personal: &'input Personal,
    /// The build date: the date a document states, and the end of every period still running.
    pub today: Date,
}

#[derive(Debug)]
pub(crate) struct Context<'input> {
    wording: Wording<'input>,
    /// What the edition's audience may print.
    pub(crate) personal: Cow<'input, Personal>,
    pub(crate) today: Date,
    pub(crate) profile: Named<'input, Profile>,
    pub(crate) experience: Experience<'input>,
}

// A document words the career like the export does, and adds the owner's details and the build date on top.
impl<'input> Deref for Context<'input> {
    type Target = Wording<'input>;

    fn deref(&self) -> &Self::Target {
        &self.wording
    }
}

impl<'input> Context<'input> {
    /// `None` when the career has no Profile.
    pub(crate) fn new(inputs: &Inputs<'input>, edition: Edition) -> Option<Self> {
        let profile = inputs.career.profiles.iter().next()?;
        Some(Self {
            wording: Wording::new(inputs.career, edition.language),
            personal: edition.audience.personal(inputs.personal),
            today: inputs.today,
            profile,
            experience: Experience::of(inputs.career, inputs.today.month()),
        })
    }

    pub(crate) fn into_missing(self) -> Vec<MissingTranslation> {
        self.wording.into_missing()
    }

    /// The Skills used within `years`, by category (issue #7).
    pub(crate) fn skill_groups(&self, years: u32) -> Vec<SkillGroup> {
        let entries = self.experience.recent(years).into_iter().map(|used| {
            let entry = SkillEntry {
                name: self.label(&used.skill.name, &used.skill.labels),
                months: used.months,
                experience: self.locale.experience(used.months),
                last_use: self.locale.month(used.last_use),
            };
            (used.skill.category, entry)
        });
        by_category(entries)
            .into_iter()
            .map(|(category, skills)| SkillGroup {
                category: category.label(self.language()).to_owned(),
                skills,
            })
            .collect()
    }

    /// From `available_from`, on `days_per_week` days.
    pub(crate) fn availability(&self) -> Option<String> {
        let (_, profile) = self.profile;
        let days_per_week = self.days_per_week();
        profile
            .available_from
            .map(|from| self.locale.availability(from, days_per_week.as_deref()))
    }

    /// `days_per_week_min` to `days_per_week_max`, as a range of the language.
    pub(crate) fn days_per_week(&self) -> Option<String> {
        let (_, profile) = self.profile;
        profile.days_per_week_min.map(|min| {
            let max = profile.days_per_week_max.unwrap_or(min);
            self.locale.range(min.get(), max.get())
        })
    }
}

pub(crate) fn url_text(url: &Url) -> String {
    url.as_str().to_owned()
}
