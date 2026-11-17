use std::fs;
use std::io;
use std::path::Path;

/// Replaces the first `from` in the note with `to`; fails when the note does not contain it, so a test never
/// breaks nothing.
pub(crate) fn replace_in(vault: &Path, note: &str, from: &str, to: &str) -> io::Result<()> {
    let path = vault.join(note);
    let text = fs::read_to_string(&path)?;
    if !text.contains(from) {
        return Err(io::Error::other(format!(
            "{note} does not contain {from:?}"
        )));
    }
    fs::write(path, text.replacen(from, to, 1))
}
