//! `kariera build` on a copy of the fixture vault, with the fictional personal details of `fixtures/personal/`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[path = "../../vault/tests/support/scratch.rs"]
mod scratch;
#[path = "../../vault/tests/support/vault_copy.rs"]
mod vault_copy;

/// A home directory whose kariera config names a fresh copy of the fixture vault, beside the fictional
/// `personal.yaml` and its photo.
fn home_with_vault_and_personal(test: &str) -> io::Result<(PathBuf, PathBuf)> {
    let vault = vault_copy::copy_of_fixture_vault(test)?;
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

fn kariera_at(home: &Path, arguments: &[&str]) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_kariera"))
        .env("HOME", home)
        .args(arguments)
        .output()
}

#[test]
fn build_writes_every_document_as_pdf_into_generated() {
    let (home, vault) = home_with_vault_and_personal("cli_build_all").unwrap();

    let output = kariera_at(&home, &["build"]).unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    for file in [
        "CV_Robin_Beispiel.pdf",
        "Lebenslauf_Robin_Beispiel.pdf",
        "職務経歴書_Robin_Beispiel.pdf",
        "スキルシート_Robin_Beispiel.pdf",
    ] {
        let path = vault.join("Generated").join(file);
        assert!(
            stdout.contains(&format!("Wrote {} (", path.display())),
            "{stdout}"
        );
        assert!(fs::read(&path).unwrap().starts_with(b"%PDF-"), "{file}");
    }
}

#[test]
fn build_warns_about_each_note_that_printed_its_english() {
    let (home, _) = home_with_vault_and_personal("cli_build_warnings").unwrap();

    let output = kariera_at(&home, &["build", "--lang", "ja"]).unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert_eq!(
        stdout
            .matches("warning: Globex - Search has no ja text")
            .count(),
        1,
        "{stdout}"
    );
    assert!(!stdout.contains("CV_"), "{stdout}");
}

#[test]
fn build_of_a_document_in_a_language_it_lacks_prints_the_usage_and_exits_two() {
    let output = Command::new(env!("CARGO_BIN_EXE_kariera"))
        .args(["build", "--doc", "cv", "--lang", "ja"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .starts_with("kariera: the cv is written in en and de only"),
    );
}
