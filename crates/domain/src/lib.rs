//! No I/O in this crate: it is the vault glossary as types, used by every other crate (ADR 0001).
//!
//! The note types mirror the tables of the vault schema field by field, under the same names. Values have the type
//! the schema gives them, so a `Month` is always a valid month. Rules between fields of one note (a conditional
//! field) are kept by the vault reader and stated on each type; rules between notes are not types.

mod calendar;
mod career;
mod contract;
mod engagement;
mod fingerprint;
mod fixed_list;
mod language;
mod link;
mod monthly_log;
mod number;
mod organization;
mod personal;
mod profile;
mod project;
mod prose;
mod qualification;
mod skill;
mod source;
mod text;
mod translatable;
mod translated;
mod value_error;

pub use calendar::{Date, Month};
pub use career::{Career, Note, NoteType, Notes};
pub use contract::{Contract, ContractType, RateStep, RateUnit};
pub use engagement::{Engagement, EngagementType};
pub use fingerprint::Fingerprint;
pub use fixed_list::{FixedList, Labelled};
pub use language::{Language, TranslationLanguage};
pub use link::{Link, NoteName};
pub use monthly_log::MonthlyLog;
pub use number::{DaysPerWeek, Hours, TeamSize, WholeNumber, Yen};
pub use organization::{Organization, OrganizationKind};
pub use personal::{Personal, Photo};
pub use profile::Profile;
pub use project::{Phase, Project};
pub use prose::{Bullets, Paragraphs};
pub use qualification::{Qualification, QualificationCategory};
pub use skill::{Skill, SkillCategory};
pub use source::{Source, SourceKind, SourcePath};
pub use text::{Text, Url};
pub use translatable::{EnglishText, Translatable};
pub use translated::{Translated, Translations};
pub use value_error::ValueError;
