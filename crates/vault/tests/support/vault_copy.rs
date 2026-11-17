use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub(crate) fn fixture_vault() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/vault")
}

/// A fresh copy of the fixture vault, for a test that breaks or writes to it. It lives in cargo's scratch directory
/// for integration tests, under `target/`; a unit test has none, so its copy lives in the system's temporary
/// directory, apart for each crate.
pub(crate) fn copy_of_fixture_vault(test: &str) -> io::Result<PathBuf> {
    let scratch = option_env!("CARGO_TARGET_TMPDIR").map_or_else(
        || std::env::temp_dir().join(concat!("kariera-", env!("CARGO_PKG_NAME"))),
        PathBuf::from,
    );
    let copy = scratch.join(test);
    if copy.exists() {
        fs::remove_dir_all(&copy)?;
    }
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
