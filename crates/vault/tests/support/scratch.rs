use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Cargo sets `CARGO_TARGET_TMPDIR` for integration tests only; a unit test falls back to the same place in this
/// checkout. Never the system's temporary directory: that one is shared with every other checkout on the machine.
fn scratch_root() -> PathBuf {
    option_env!("CARGO_TARGET_TMPDIR").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp"),
        PathBuf::from,
    )
}

/// The process id keeps two runs of one checkout (a test run beside a mutation run, or nextest's one process per
/// test) apart; a directory left by an earlier process with the same id is replaced.
pub(crate) fn fresh_directory(name: &str) -> io::Result<PathBuf> {
    let directory = scratch_root()
        .join(format!("kariera-{}", std::process::id()))
        .join(name);
    if directory.exists() {
        fs::remove_dir_all(&directory)?;
    }
    fs::create_dir_all(&directory)?;
    Ok(directory)
}
