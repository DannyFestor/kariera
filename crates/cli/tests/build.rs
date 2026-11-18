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
fn build_writes_the_skill_sheet_as_xlsx_beside_its_pdf() {
    let (home, vault) = home_with_vault_and_personal("cli_build_skill_sheet_xlsx").unwrap();

    let output = kariera_at(&home, &["build"]).unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    let path = vault.join("Generated/スキルシート_Robin_Beispiel.xlsx");
    assert!(
        stdout.contains(&format!("Wrote {}\n", path.display())),
        "{stdout}"
    );
    assert!(fs::read(&path).unwrap().starts_with(b"PK"));
    let xlsx_files: Vec<_> = fs::read_dir(vault.join("Generated"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .filter(|name| name.to_string_lossy().ends_with(".xlsx"))
        .collect();
    assert_eq!(xlsx_files.len(), 1, "{xlsx_files:?}");
}

#[test]
fn build_writes_the_cv_and_the_shokumu_keirekisho_as_docx_beside_the_pdfs() {
    let (home, vault) = home_with_vault_and_personal("cli_build_docx").unwrap();

    let output = kariera_at(&home, &["build"]).unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    for file in [
        "CV_Robin_Beispiel.docx",
        "Lebenslauf_Robin_Beispiel.docx",
        "職務経歴書_Robin_Beispiel.docx",
    ] {
        let path = vault.join("Generated").join(file);
        assert!(
            stdout.contains(&format!("Wrote {}\n", path.display())),
            "{stdout}"
        );
        assert!(fs::read(&path).unwrap().starts_with(b"PK"), "{file}");
    }
    assert!(
        !vault
            .join("Generated/スキルシート_Robin_Beispiel.docx")
            .exists()
    );
}

#[test]
fn a_second_build_writes_the_same_docx_bytes_whatever_was_built_before_it() {
    let (home, vault) = home_with_vault_and_personal("cli_build_docx_twice").unwrap();
    let docx = vault.join("Generated/Lebenslauf_Robin_Beispiel.docx");
    kariera_at(&home, &["build"]).unwrap();
    let after_everything = fs::read(&docx).unwrap();

    fs::remove_file(&docx).unwrap();
    kariera_at(&home, &["build", "--doc", "cv", "--lang", "de"]).unwrap();

    assert!(fs::read(&docx).unwrap() == after_everything);
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
