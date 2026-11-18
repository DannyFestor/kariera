use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[path = "../../vault/tests/support/scratch.rs"]
mod scratch;

use scratch::fresh_directory;

const GO: &str = "---\ntype: skill\nname: Go\ncategory: language\nreviewed: false\n---\n";

fn fixture_vault() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/vault")
}

/// A home directory whose config names `vault`.
fn home_for(test: &str, vault: &Path) -> io::Result<PathBuf> {
    let home = fresh_directory(&format!("{test}_home"))?;
    let config = home.join(".config/kariera");
    fs::create_dir_all(&config)?;
    fs::write(
        config.join("config.toml"),
        format!("vault = {:?}\n", vault.display().to_string()),
    )?;
    Ok(home)
}

/// A vault of one Skill note, which `Vault::load` accepts: the checks across notes belong to validation.
fn vault_with_go(test: &str) -> io::Result<PathBuf> {
    let vault = fresh_directory(&format!("{test}_vault"))?;
    fs::create_dir_all(vault.join("Skills"))?;
    fs::write(vault.join("Skills/Go.md"), GO)?;
    Ok(vault)
}

fn kariera(home: &Path, arguments: &[&str]) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_kariera"))
        .env("HOME", home)
        .args(arguments)
        .output()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn stale_lists_every_empty_and_differing_translation_and_exits_zero() {
    let home = home_for("cli_stale_fixture", &fixture_vault()).unwrap();

    let output = kariera(&home, &["translations", "--stale"]).unwrap();

    assert_eq!(output.status.code(), Some(0));
    let lines: Vec<String> = stdout(&output).lines().map(str::to_owned).collect();
    assert!(lines.contains(&"Go\tja\tempty".to_owned()), "{lines:?}");
    assert!(lines.contains(&"Go\tde\tempty".to_owned()), "{lines:?}");
    assert!(
        lines.contains(&"Acme - Shop\tja\tdiffers".to_owned()),
        "{lines:?}"
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.starts_with("2021-04 Acme Contract"))
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn marking_a_translation_takes_it_off_the_stale_list() {
    let vault = vault_with_go("cli_mark").unwrap();
    let home = home_for("cli_mark", &vault).unwrap();

    let marked = kariera(&home, &["translations", "--mark", "Go", "--lang", "ja"]).unwrap();
    let stale = kariera(&home, &["translations", "--stale"]).unwrap();

    assert_eq!(marked.status.code(), Some(0), "{}", stderr(&marked));
    assert_eq!(
        stdout(&marked),
        "Marked Go: ja is translated from 007a4f991fa0\n"
    );
    assert_eq!(stdout(&stale), "Go\tde\tempty\n");
    assert_eq!(
        fs::read_to_string(vault.join("Skills/Go.md")).unwrap(),
        GO.replace(
            "reviewed: false\n",
            "translated_from:\n  ja: 007a4f991fa0\nreviewed: false\n"
        )
    );
}

#[test]
fn marking_a_note_that_does_not_exist_fails() {
    let vault = vault_with_go("cli_mark_missing").unwrap();
    let home = home_for("cli_mark_missing", &vault).unwrap();

    let output = kariera(&home, &["translations", "--mark", "Rust", "--lang", "de"]).unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("no note is named \"Rust\""),
        "{}",
        stderr(&output)
    );
    assert_eq!(stdout(&output), "");
}

#[test]
fn a_missing_config_fails_and_names_the_config() {
    let home = fresh_directory("cli_no_config_home").unwrap();

    let output = kariera(&home, &["translations", "--stale"]).unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("config.toml"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_mark_without_a_language_is_a_usage_error() {
    let home = home_for("cli_mark_usage", &fixture_vault()).unwrap();

    let output = kariera(&home, &["translations", "--mark", "Go"]).unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr(&output).contains("--mark needs --lang <ja|de>"),
        "{}",
        stderr(&output)
    );
}
