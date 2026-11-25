//! `kariera export --json` on a copy of the fixture vault, beside the fictional `personal.yaml` it must not read.

use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Output;

#[path = "support/home.rs"]
mod home;
#[path = "support/json_schema.rs"]
mod json_schema;
#[path = "../../vault/tests/support/replace_in.rs"]
mod replace_in_support;
#[path = "../../vault/tests/support/scratch.rs"]
mod scratch;
#[path = "../../vault/tests/support/vault_copy.rs"]
mod vault_copy;

use home::{home_with_vault_and_personal, kariera_at};
use replace_in_support::replace_in;
use serde_json::Value;

const ACME: &str = "Engagements/2021-04 Acme.md";
const LEDGER: &str = "Projects/Acme - Ledger.md";

// clippy.toml allows unwrap inside #[test] functions only, so the helpers return errors for the tests to unwrap.
fn export(home: &Path) -> io::Result<Output> {
    kariera_at(home, &["export", "--json"])
}

/// Lossy, so a stray byte shows up in the comparison instead of failing the helper.
fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn mark_reviewed(vault: &Path, note: &str) -> io::Result<()> {
    replace_in(vault, note, "reviewed: false", "reviewed: true")
}

fn schema() -> Result<Value, Box<dyn Error>> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemas/career-export.schema.json");
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

#[test]
fn export_prints_the_public_career_of_the_fixture_vault() {
    let (home, _) = home_with_vault_and_personal("cli_export").unwrap();

    let output = export(&home).unwrap();

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(stderr(&output), "");
    insta::assert_snapshot!("fixture-export", stdout(&output));
}

#[test]
fn the_export_is_the_same_bytes_every_time_and_ends_with_a_newline() {
    let (home, _) = home_with_vault_and_personal("cli_export_twice").unwrap();

    let first = stdout(&export(&home).unwrap());
    let second = stdout(&export(&home).unwrap());

    assert_eq!(first, second);
    assert!(first.ends_with("}\n"), "{first}");
}

#[test]
fn the_export_of_the_fixture_vault_matches_the_committed_json_schema() {
    let (home, vault) = home_with_vault_and_personal("cli_export_schema").unwrap();
    mark_reviewed(&vault, ACME).unwrap();
    mark_reviewed(&vault, LEDGER).unwrap();

    let output = export(&home).unwrap();

    let exported: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(
        json_schema::violations(&schema().unwrap(), &exported),
        Vec::<String>::new()
    );
    assert_eq!(exported["projects"].as_array().unwrap().len(), 3);
}

#[test]
fn the_schema_refuses_an_export_that_breaks_its_contract() {
    let (home, _) = home_with_vault_and_personal("cli_export_schema_refuses").unwrap();
    let mut exported: Value = serde_json::from_str(&stdout(&export(&home).unwrap())).unwrap();
    exported["engagements"][0]["start"] = Value::from("April 2019");
    exported["engagements"][0]["rate"] = Value::from(5000);
    exported["profile"]
        .as_object_mut()
        .unwrap()
        .remove("headline");

    let violations = json_schema::violations(&schema().unwrap(), &exported);

    assert_eq!(
        violations,
        [
            "/engagements/0: the value has the key \"rate\", which the schema does not declare",
            "/engagements/0/start: the value does not match \"^[0-9]{4}-(0[1-9]|1[0-2])$\"",
            "/profile: the value lacks the required key \"headline\"",
        ]
    );
}

#[test]
fn an_unreviewed_engagement_is_left_out_and_its_reviewed_projects_with_it() {
    let (home, vault) = home_with_vault_and_personal("cli_export_unreviewed").unwrap();

    let before = stdout(&export(&home).unwrap());
    mark_reviewed(&vault, ACME).unwrap();
    let after = stdout(&export(&home).unwrap());

    assert!(!before.contains("2021-04 Acme"), "{before}");
    assert!(!before.contains("Acme - Shop"), "{before}");
    assert!(
        after.contains("\"engagement\": \"2021-04 Acme\""),
        "{after}"
    );
    assert!(after.contains("\"note\": \"Acme - Shop\""), "{after}");
}

#[test]
fn a_confidential_project_exports_its_public_title_and_no_end_client() {
    let (home, vault) = home_with_vault_and_personal("cli_export_confidential").unwrap();
    mark_reviewed(&vault, ACME).unwrap();
    mark_reviewed(&vault, LEDGER).unwrap();

    let output = export(&home).unwrap();

    let exported = stdout(&output);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(
        exported.contains("\"title\": \"Accounting platform for a logistics company\""),
        "{exported}"
    );
    for hidden in ["Northwind", "freight ledger", "end_client"] {
        assert!(!exported.contains(hidden), "{hidden} in {exported}");
    }
}

#[test]
fn a_confidential_project_whose_title_is_its_public_title_exports() {
    let (home, vault) = home_with_vault_and_personal("cli_export_title_is_public").unwrap();
    mark_reviewed(&vault, ACME).unwrap();
    mark_reviewed(&vault, LEDGER).unwrap();
    replace_in(
        &vault,
        LEDGER,
        "en: Northwind freight ledger",
        "en: Accounting platform for a logistics company",
    )
    .unwrap();

    let output = export(&home).unwrap();

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).contains("\"title\": \"Accounting platform for a logistics company\""),
        "{}",
        stdout(&output)
    );
}

#[test]
fn an_end_client_named_in_a_note_name_fails_the_export_and_prints_nothing() {
    let (home, vault) = home_with_vault_and_personal("cli_export_guard").unwrap();
    mark_reviewed(&vault, ACME).unwrap();
    mark_reviewed(&vault, LEDGER).unwrap();
    fs::rename(
        vault.join(LEDGER),
        vault.join("Projects/Acme - Northwind Ledger.md"),
    )
    .unwrap();

    let output = export(&home).unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        "The export would give away a confidential Project, so it was not written:\n\
         \"Acme - Northwind Ledger\" gives away the End Client of the confidential Project [[Acme - Northwind Ledger]]\n"
    );
}

#[test]
fn a_title_alias_or_label_named_in_a_note_name_fails_the_export() {
    let cases = [
        ("Acme - Northwind freight ledger", "the title of"),
        ("Acme - ノースウィンド運輸", "the End Client of"),
        ("Acme - Nordwind Spedition", "the End Client of"),
    ];
    for (index, (note, gives_away)) in cases.into_iter().enumerate() {
        let (home, vault) =
            home_with_vault_and_personal(&format!("cli_export_guard_{index}")).unwrap();
        mark_reviewed(&vault, ACME).unwrap();
        mark_reviewed(&vault, LEDGER).unwrap();
        replace_in(
            &vault,
            "Organizations/Northwind.md",
            "  - end-client\n",
            "  - end-client\nlabels:\n  de: Nordwind Spedition\n",
        )
        .unwrap();
        fs::rename(
            vault.join(LEDGER),
            vault.join(format!("Projects/{note}.md")),
        )
        .unwrap();

        let output = export(&home).unwrap();

        assert_eq!(output.status.code(), Some(1), "{note}");
        assert_eq!(stdout(&output), "", "{note}");
        assert!(
            stderr(&output).contains(&format!(
                "{note:?} gives away {gives_away} the confidential Project [[{note}]]"
            )),
            "{}",
            stderr(&output)
        );
    }
}

#[test]
fn a_vault_that_fails_validation_fails_the_export_and_prints_nothing() {
    let (home, vault) = home_with_vault_and_personal("cli_export_invalid").unwrap();
    replace_in(&vault, "Skills/Go.md", "category: language\n", "").unwrap();

    let output = export(&home).unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        "The vault has 1 error, so nothing was exported: `kariera validate` lists it.\n"
    );
}

#[test]
fn no_translation_rate_hours_or_personal_detail_is_exported() {
    let (home, vault) = home_with_vault_and_personal("cli_export_private").unwrap();
    mark_reviewed(&vault, ACME).unwrap();
    mark_reviewed(&vault, LEDGER).unwrap();

    let output = export(&home).unwrap();

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let exported = stdout(&output);
    let japanese = exported
        .chars()
        .find(|&character| matches!(character, '\u{3040}'..='\u{30ff}' | '\u{4e00}'..='\u{9fff}'));
    assert_eq!(japanese, None, "{exported}");
    let private = [
        // German translations of exported notes.
        "Full-Stack-Entwickler",
        "Softwareentwickler",
        "Sendungsverfolgung",
        "Verhandlungssicher",
        "Bachelor Informatik",
        // Contracts, Monthly Logs, Sources and the fields no document needs.
        "rate",
        "5000",
        "hours",
        "152.5",
        "sources",
        ".scratch",
        "contract",
        "translated_from",
        "reviewed",
        "available_from",
        "days_per_week",
        "2026-04-01",
        // personal.yaml, its public map included.
        "robin@example.com",
        "hello@example.com",
        "+81 90",
        "Sapporo",
        "1990-01-01",
        "001-0000",
    ];
    for text in private {
        assert!(!exported.contains(text), "{text} in {exported}");
    }
}

#[test]
fn export_without_json_is_a_usage_error() {
    let (home, _) = home_with_vault_and_personal("cli_export_usage").unwrap();

    let output = kariera_at(&home, &["export"]).unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr(&output).starts_with("kariera: export needs --json\n"),
        "{}",
        stderr(&output)
    );
}
