use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A home directory whose kariera config names a fresh copy of the fixture vault, beside the fictional
/// `personal.yaml` and its photo.
pub(crate) fn home_with_vault_and_personal(test: &str) -> io::Result<(PathBuf, PathBuf)> {
    let vault = crate::vault_copy::copy_of_fixture_vault(test)?;
    let home = vault.with_extension("home");
    let config = home.join(".config/kariera");
    fs::create_dir_all(&config)?;
    fs::write(
        config.join("config.toml"),
        format!("vault = {:?}\n", vault.display().to_string()),
    )?;
    let personal = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/personal");
    fs::copy(
        personal.join("fictional-personal.yaml"),
        config.join("personal.yaml"),
    )?;
    fs::copy(personal.join("photo.png"), config.join("photo.png"))?;
    Ok((home, vault))
}

pub(crate) fn kariera_at(home: &Path, arguments: &[&str]) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_kariera"))
        .env("HOME", home)
        .args(arguments)
        .output()
}
