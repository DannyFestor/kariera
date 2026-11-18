use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub(crate) fn fixture_vault() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/vault")
}

/// A fresh copy of the fixture vault, for a test that breaks or writes to it.
pub(crate) fn copy_of_fixture_vault(test: &str) -> io::Result<PathBuf> {
    let copy = super::scratch::fresh_directory(test)?;
    copy_directory(&fixture_vault(), &copy)?;
    Ok(copy)
}

fn copy_directory(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_directory(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
