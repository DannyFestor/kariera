//! Reads the vault's notes into domain types (ADR 0001).
//!
//! A note is read in two steps. First on its own: its name, its frontmatter as YAML, its `type` and its body
//! sections. Then, once every note name is known, its fields and sections against its type's schema table, with
//! every link resolved. Every problem of every note is collected, with file, line and field.
//!
//! What one note's frontmatter alone can tell is checked here: required, conditional and unknown fields, each value
//! against its field type, links against the names and types of the other notes, and the body's headings. Rules
//! between values (an `end` before its `start`, a `rate_history` that does not end at `rate`), between notes, and
//! about file names, folders and formatting are left to validation.

mod body;
mod config;
mod creation;
mod cursor;
mod discovery;
mod error;
mod fields;
mod fingerprint;
mod names;
mod notes;
mod problems;
mod raw_note;
mod sections;
mod translated_from;
mod translations;
mod value;
mod vault;
mod write;
mod yaml;

pub use body::{Body, Line, Section};
pub use config::{Config, ConfigError};
pub use creation::{CreateError, NewEngagement, NewLog, NewNote, NewProject, OrganizationRole};
pub use error::{LoadError, Location, NoteError, Problem};
pub use translated_from::EditError;
pub use translations::{MarkError, Translation};
pub use vault::{NoteFile, Vault};
