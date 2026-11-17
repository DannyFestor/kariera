use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// The note is replaced in one step, so a crash or a full disk leaves the old note and not half of the new one.
/// The vault has no git to bring a damaged note back.
pub(crate) fn replace_file(path: &Path, text: &str) -> io::Result<()> {
    let permissions = fs::metadata(path)?.permissions();
    let temporary = temporary_sibling(path);
    let written = fs::write(&temporary, text)
        .and_then(|()| fs::set_permissions(&temporary, permissions))
        .and_then(|()| fs::rename(&temporary, path));
    if written.is_err() {
        drop(fs::remove_file(&temporary));
    }
    written
}

/// `create_new` fails if the file exists, so a race with another writer cannot overwrite either.
pub(crate) fn create_file(path: &Path, text: &str) -> io::Result<()> {
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(text.as_bytes())
}

/// In the same folder, because a rename cannot cross file systems. Not a note: it does not end in `.md`.
fn temporary_sibling(path: &Path) -> PathBuf {
    let name = path.file_name().and_then(OsStr::to_str).unwrap_or_default();
    path.with_file_name(format!(".{name}.kariera-tmp"))
}
