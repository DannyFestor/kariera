use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::LoadError;

/// The folders that hold notes (schema §1.1), searched with their subfolders. Nothing else in the vault is read:
/// not `Templates/`, `Generated/`, `docs/` or `.scratch/`, and no other Markdown file.
const NOTE_FOLDERS: [&str; 7] = [
    "Organizations",
    "Engagements",
    "Contracts",
    "Projects",
    "Monthly Logs",
    "Skills",
    "Qualifications",
];

const PROFILE: &str = "Profile.md";

/// Relative to `root`, sorted, so the vault reads the same on every machine.
pub(crate) fn note_paths(root: &Path) -> Result<Vec<PathBuf>, LoadError> {
    fs::read_dir(root).map(drop).map_err(unreadable(root))?;
    let mut paths = Vec::new();
    for folder in NOTE_FOLDERS {
        collect(root, Path::new(folder), &mut paths)?;
    }
    if root.join(PROFILE).is_file() {
        paths.push(PathBuf::from(PROFILE));
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
        let relative = folder.join(entry.file_name());
        let file_type = entry.file_type().map_err(unreadable(&entry.path()))?;
        if is_hidden(&relative) {
            continue;
        }
        if file_type.is_dir() {
            collect(root, &relative, paths)?;
        } else if file_type.is_file() && relative.extension() == Some(OsStr::new("md")) {
            paths.push(relative);
        }
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
