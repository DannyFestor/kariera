use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use kariera_domain::{Career, NoteName, NoteType};

use crate::body::Body;
use crate::discovery::{note_paths, other_markdown_paths};
use crate::names::Names;
use crate::notes::{self, NoteSource};
use crate::raw_note::RawNote;
use crate::value::scalar_text;
use crate::yaml::Node;
use crate::{LoadError, Location, NoteError, Problem};

/// The whole vault, read and with every link resolved.
#[derive(Debug)]
pub struct Vault {
    root: PathBuf,
    career: Career,
    notes: Vec<NoteFile>,
}

/// What `Vault::inspect` found: the notes that read, and the problems of those that did not.
#[derive(Debug)]
pub struct Inspection {
    pub vault: Vault,
    pub errors: Vec<NoteError>,
}

/// A note as it is on disk, for the checks and edits that need more than its domain value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteFile {
    /// Relative to the vault root.
    pub path: PathBuf,
    pub name: NoteName,
    pub note_type: NoteType,
    /// The whole file, frontmatter included.
    pub text: String,
    pub frontmatter: Node,
    pub body: Body,
}

/// A Markdown file outside the note folders that declares a note `type` in its frontmatter, so it is a note in
/// the wrong place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stray {
    /// Relative to the vault root.
    pub path: PathBuf,
    /// The `type` as written, which need not be a note type.
    pub declared_type: String,
}

impl Vault {
    /// Reads every note. Fails with every problem of every note, or when a file cannot be read at all.
    pub fn load(root: &Path) -> Result<Self, LoadError> {
        let Inspection { vault, errors } = Self::inspect(root)?;
        if errors.is_empty() {
            Ok(vault)
        } else {
            Err(LoadError::InvalidNotes(errors))
        }
    }

    /// Reads every note and keeps going past the notes that do not read: the vault holds the notes that did, the
    /// errors the ones that did not. Links to a note that did not read still resolve.
    pub fn inspect(root: &Path) -> Result<Inspection, LoadError> {
        let mut raw_notes = read_notes(root)?;
        let names = index_names(&mut raw_notes);
        let mut career = Career::default();
        let mut notes = Vec::new();
        let mut errors = Vec::new();
        for raw_note in raw_notes {
            match add_note(&mut career, raw_note, &names) {
                Ok(note) => notes.push(note),
                Err(note_errors) => errors.extend(note_errors),
            }
        }
        let vault = Self {
            root: root.to_owned(),
            career,
            notes,
        };
        Ok(Inspection { vault, errors })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub const fn career(&self) -> &Career {
        &self.career
    }

    /// In the order of their paths.
    #[must_use]
    pub fn notes(&self) -> &[NoteFile] {
        &self.notes
    }

    /// Markdown files that declare a `type` but are not in a note folder. Markdown without a `type` is not a note
    /// (agent instructions, a glossary, a to-do list) and `Templates/`, `Generated/`, `docs/` and `.scratch/`
    /// are exempt (schema §1.1).
    pub fn strays(&self) -> Result<Vec<Stray>, LoadError> {
        let mut strays = Vec::new();
        for path in other_markdown_paths(&self.root)? {
            let absolute = self.root.join(&path);
            let bytes = fs::read(&absolute).map_err(|source| LoadError::Unreadable {
                path: absolute,
                source,
            })?;
            if let Some(declared_type) = declared_type(&RawNote::read(path.clone(), bytes)) {
                strays.push(Stray {
                    path,
                    declared_type,
                });
            }
        }
        Ok(strays)
    }
}

fn declared_type(raw_note: &RawNote) -> Option<String> {
    let node = raw_note.frontmatter.as_ref()?.get("type")?;
    scalar_text(node)
        .ok()
        .filter(|_| !node.is_empty())
        .map(str::to_owned)
}

fn read_notes(root: &Path) -> Result<Vec<RawNote>, LoadError> {
    note_paths(root)?
        .into_iter()
        .map(|path| {
            let absolute = root.join(&path);
            let bytes = fs::read(&absolute).map_err(|source| LoadError::Unreadable {
                path: absolute,
                source,
            })?;
            Ok(RawNote::read(path, bytes))
        })
        .collect()
}

/// A link names its target, so two notes of one name cannot both be linked: the later one is a problem.
fn index_names(raw_notes: &mut [RawNote]) -> Names {
    let mut names = Names::default();
    let mut first_paths: BTreeMap<NoteName, PathBuf> = BTreeMap::new();
    for note in raw_notes {
        let Some(name) = note.name.clone() else {
            continue;
        };
        if let Some(first_path) = first_paths.get(&name) {
            let message = format!("has the same name as {}", first_path.display());
            note.problems
                .push(Problem::new(Location::Note, None, message));
        } else {
            first_paths.insert(name.clone(), note.path.clone());
            names.insert(name, note.note_type);
        }
    }
    names
}

fn add_note(
    career: &mut Career,
    raw_note: RawNote,
    names: &Names,
) -> Result<NoteFile, Vec<NoteError>> {
    let RawNote {
        path,
        name,
        text,
        frontmatter,
        note_type,
        body,
        problems,
    } = raw_note;
    let (Some(name), Some(frontmatter), Some(note_type), true) =
        (name, frontmatter, note_type, problems.is_empty())
    else {
        return Err(note_errors(&path, problems));
    };
    let source = NoteSource {
        frontmatter: &frontmatter,
        body: &body,
        names,
    };
    notes::add(career, name.clone(), note_type, source)
        .map_err(|parse_problems| note_errors(&path, parse_problems))?;
    Ok(NoteFile {
        path,
        name,
        note_type,
        text,
        frontmatter,
        body,
    })
}

fn note_errors(path: &Path, problems: Vec<Problem>) -> Vec<NoteError> {
    problems
        .into_iter()
        .map(|problem| NoteError {
            path: path.to_owned(),
            problem,
        })
        .collect()
}
