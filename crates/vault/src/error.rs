use std::fmt;
use std::io;
use std::path::PathBuf;

use thiserror::Error;

/// Where in a note a problem sits.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Location {
    /// The note as a whole: its name, its encoding, its body as a whole.
    Note,
    /// The frontmatter block as a whole: its delimiters and its YAML.
    Frontmatter,
    /// A frontmatter field, as a path such as `sources[1].kind`.
    Field(String),
    /// A body section, as its heading such as `## 日本語 > ### Summary`.
    Section(String),
}

/// One thing wrong with one note. `message` completes a sentence that starts with the location.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Problem {
    pub location: Location,
    /// 1-based, counted from the first line of the file.
    pub line: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NoteError {
    /// Relative to the vault root.
    pub path: PathBuf,
    pub problem: Problem,
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("cannot read {}: {source}", path.display())]
    Unreadable { path: PathBuf, source: io::Error },
    #[error("{} problems in the vault:\n{}", .0.len(), lines(.0))]
    InvalidNotes(Vec<NoteError>),
}

impl Problem {
    pub(crate) fn new(location: Location, line: Option<usize>, message: impl Into<String>) -> Self {
        Self {
            location,
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Note => formatter.write_str("the note"),
            Self::Frontmatter => formatter.write_str("the frontmatter"),
            Self::Field(path) => write!(formatter, "field `{path}`"),
            Self::Section(heading) => write!(formatter, "section `{heading}`"),
        }
    }
}

impl fmt::Display for NoteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.path.display())?;
        if let Some(line) = self.problem.line {
            write!(formatter, ":{line}")?;
        }
        write!(
            formatter,
            ": {} {}",
            self.problem.location, self.problem.message
        )
    }
}

fn lines(errors: &[NoteError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{LoadError, Location, NoteError, Problem};

    fn error(location: Location, line: Option<usize>) -> NoteError {
        NoteError {
            path: PathBuf::from("Projects/Acme - Shop.md"),
            problem: Problem::new(location, line, "is required"),
        }
    }

    #[test]
    fn a_field_error_names_file_line_and_field() {
        let error = error(Location::Field("title.en".to_owned()), Some(7));

        assert_eq!(
            error.to_string(),
            "Projects/Acme - Shop.md:7: field `title.en` is required"
        );
    }

    #[test]
    fn a_section_error_names_the_heading() {
        let error = error(Location::Section("## Summary".to_owned()), Some(30));

        assert_eq!(
            error.to_string(),
            "Projects/Acme - Shop.md:30: section `## Summary` is required"
        );
    }

    #[test]
    fn a_note_error_without_a_line_names_the_file() {
        let error = error(Location::Note, None);

        assert_eq!(
            error.to_string(),
            "Projects/Acme - Shop.md: the note is required"
        );
    }

    #[test]
    fn a_frontmatter_error_names_the_frontmatter() {
        let error = error(Location::Frontmatter, Some(2));

        assert_eq!(
            error.to_string(),
            "Projects/Acme - Shop.md:2: the frontmatter is required"
        );
    }

    #[test]
    fn invalid_notes_are_listed_one_per_line() {
        let errors = vec![error(Location::Note, None), error(Location::Note, Some(1))];

        let message = LoadError::InvalidNotes(errors).to_string();

        assert_eq!(
            message,
            "2 problems in the vault:\nProjects/Acme - Shop.md: the note is required\n\
             Projects/Acme - Shop.md:1: the note is required"
        );
    }
}
