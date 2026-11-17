use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use kariera_domain::{FixedList, Month, NoteType};

use crate::LoadError;

/// The folder under the vault root for the notes of a type (schema §1.1).
///
/// Empty for the Profile, which lives in the vault root itself. Monthly Logs lie one level further down, see
/// `monthly_log_folder`.
#[must_use]
pub const fn note_folder(note_type: NoteType) -> &'static str {
    match note_type {
        NoteType::Organization => "Organizations",
        NoteType::Engagement => "Engagements",
        NoteType::Contract => "Contracts",
        NoteType::Project => "Projects",
        NoteType::MonthlyLog => "Monthly Logs",
        NoteType::Skill => "Skills",
        NoteType::Qualification => "Qualifications",
        NoteType::Profile => "",
    }
}

/// A Monthly Log lies in a folder per year (schema §1.1).
#[must_use]
pub fn monthly_log_folder(month: Month) -> PathBuf {
    Path::new(note_folder(NoteType::MonthlyLog)).join(format!("{:04}", month.year()))
}

/// The folders that hold notes, searched with their subfolders. Nothing else in the vault is read as a note:
/// not `Templates/`, `Generated/`, `docs/` or `.scratch/`, and no other Markdown file.
fn note_folders() -> impl Iterator<Item = &'static str> {
    NoteType::ALL
        .iter()
        .map(|note_type| note_folder(*note_type))
        .filter(|folder| !folder.is_empty())
}

const PROFILE: &str = "Profile.md";

/// Folders at the vault root that hold no notes at all (schema §1.1).
const EXEMPT_FOLDERS: [&str; 4] = ["Templates", "Generated", "docs", ".scratch"];

/// Relative to `root`, sorted, so the vault reads the same on every machine.
pub(crate) fn note_paths(root: &Path) -> Result<Vec<PathBuf>, LoadError> {
    fs::read_dir(root).map(drop).map_err(unreadable(root))?;
    let mut paths = Vec::new();
    for folder in note_folders() {
        collect(root, Path::new(folder), &mut paths)?;
    }
    if root.join(PROFILE).is_file() {
        paths.push(PathBuf::from(PROFILE));
    }
    paths.sort();
    Ok(paths)
}

/// Relative to `root`, sorted: every Markdown file that is neither in a note folder, nor exempt, nor `Profile.md`.
/// Whether such a file is a note in the wrong place depends on its frontmatter.
pub(crate) fn other_markdown_paths(root: &Path) -> Result<Vec<PathBuf>, LoadError> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(root).map_err(unreadable(root))? {
        let entry = entry.map_err(unreadable(root))?;
        let name = entry.file_name();
        let is_note_place = name == PROFILE
            || note_folders().any(|folder| name == folder)
            || EXEMPT_FOLDERS.iter().any(|folder| name == *folder);
        if !is_note_place {
            visit(root, Path::new(""), &entry, &mut paths)?;
        }
    }
    paths.sort();
    Ok(paths)
}

/// A missing folder holds no notes yet.
fn collect(root: &Path, folder: &Path, paths: &mut Vec<PathBuf>) -> Result<(), LoadError> {
    let directory = root.join(folder);
    if !directory.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(&directory).map_err(unreadable(&directory))? {
        let entry = entry.map_err(unreadable(&directory))?;
        visit(root, folder, &entry, paths)?;
    }
    Ok(())
}

fn visit(
    root: &Path,
    folder: &Path,
    entry: &fs::DirEntry,
    paths: &mut Vec<PathBuf>,
) -> Result<(), LoadError> {
    let relative = folder.join(entry.file_name());
    let file_type = entry.file_type().map_err(unreadable(&entry.path()))?;
    if is_hidden(&relative) {
        return Ok(());
    }
    if file_type.is_dir() {
        collect(root, &relative, paths)?;
    } else if file_type.is_file() && relative.extension() == Some(OsStr::new("md")) {
        paths.push(relative);
    }
    Ok(())
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.starts_with('.'))
}

fn unreadable(path: &Path) -> impl FnOnce(io::Error) -> LoadError {
    let path = path.to_owned();
    move |source| LoadError::Unreadable { path, source }
}

#[cfg(test)]
mod tests {
    use kariera_domain::{FixedList, NoteType};

    use super::{note_folder, note_folders};

    #[test]
    fn every_note_type_but_the_profile_has_a_folder() {
        let folders: Vec<&str> = NoteType::ALL.iter().map(|t| note_folder(*t)).collect();

        assert_eq!(
            folders,
            [
                "Organizations",
                "Engagements",
                "Contracts",
                "Projects",
                "Monthly Logs",
                "Skills",
                "Qualifications",
                "",
            ]
        );
        assert_eq!(note_folders().count(), 7);
    }
}
