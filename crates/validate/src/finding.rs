use std::fmt;
use std::path::{Path, PathBuf};

use kariera_vault::{Location, NoteError, NoteFile};

use crate::field_path::FieldPath;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

/// Which part of the schema a finding is about, for readers that sort or filter findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
    /// What the vault reader checks on a note alone: fields, values, links, body sections.
    Schema,
    Folder,
    FileName,
    DuplicateName,
    FieldOrder,
    YamlStyle,
    Period,
    LinkRole,
    Contract,
    Rate,
    Hours,
    Alias,
    Body,
    Source,
    Confidentiality,
    ForbiddenContent,
    Profile,
}

impl Rule {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Schema => "schema",
            Self::Folder => "folder",
            Self::FileName => "file-name",
            Self::DuplicateName => "duplicate-name",
            Self::FieldOrder => "field-order",
            Self::YamlStyle => "yaml-style",
            Self::Period => "period",
            Self::LinkRole => "link-role",
            Self::Contract => "contract",
            Self::Rate => "rate",
            Self::Hours => "hours",
            Self::Alias => "alias",
            Self::Body => "body",
            Self::Source => "source",
            Self::Confidentiality => "confidentiality",
            Self::ForbiddenContent => "forbidden-content",
            Self::Profile => "profile",
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.path.display())?;
        if let Some(line) = self.line {
            write!(formatter, ":{line}")?;
        }
        write!(
            formatter,
            ": {} [{}] {} {}",
            self.severity.name(),
            self.rule.code(),
            self.location,
            self.message
        )
    }
}

/// One thing wrong with one note. `message` completes a sentence that starts with the location.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    /// Relative to the vault root.
    pub path: PathBuf,
    /// 1-based, counted from the first line of the file.
    pub line: Option<usize>,
    pub location: Location,
    pub severity: Severity,
    pub rule: Rule,
    pub message: String,
}

/// Where in a note a finding sits; the finding gets its line from the note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum At {
    Note,
    Field(FieldPath),
    Section { heading: String, line: usize },
}

impl Finding {
    /// An error; `warning` turns it into a warning.
    pub(crate) fn new(file: &NoteFile, rule: Rule, at: At, message: impl Into<String>) -> Self {
        let (location, line) = match at {
            At::Note => (Location::Note, None),
            At::Field(path) => {
                let line = path.line_in(&file.frontmatter);
                (Location::Field(path.to_string()), Some(line))
            }
            At::Section { heading, line } => (Location::Section(heading), Some(line)),
        };
        Self {
            path: file.path.clone(),
            line,
            location,
            severity: Severity::Error,
            rule,
            message: message.into(),
        }
    }

    pub(crate) fn at_path(path: &Path, rule: Rule, message: impl Into<String>) -> Self {
        Self {
            path: path.to_owned(),
            line: None,
            location: Location::Note,
            severity: Severity::Error,
            rule,
            message: message.into(),
        }
    }

    #[must_use]
    pub(crate) const fn warning(mut self) -> Self {
        self.severity = Severity::Warning;
        self
    }
}

impl From<NoteError> for Finding {
    fn from(error: NoteError) -> Self {
        Self {
            path: error.path,
            line: error.problem.line,
            location: error.problem.location,
            severity: Severity::Error,
            rule: Rule::Schema,
            message: error.problem.message,
        }
    }
}
