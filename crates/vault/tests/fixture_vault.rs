use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use kariera_domain::{FixedList, Link, NoteType, Organization};
use kariera_vault::{LoadError, Location, NoteError, Problem, Vault};

#[path = "support/vault_copy.rs"]
mod vault_copy;

use vault_copy::{copy_of_fixture_vault, fixture_vault};

fn replace_in(vault: &Path, note: &str, from: &str, to: &str) -> io::Result<()> {
    let path = vault.join(note);
    let text = fs::read_to_string(&path)?;
    if !text.contains(from) {
        return Err(io::Error::other(format!(
            "{note} does not contain {from:?}"
        )));
    }
    fs::write(path, text.replacen(from, to, 1))
}

/// The problems of a vault that must not load.
fn errors_of(vault: &Path) -> Result<Vec<NoteError>, String> {
    match Vault::load(vault) {
        Err(LoadError::InvalidNotes(errors)) => Ok(errors),
        Err(other) => Err(format!("the vault could not be read at all: {other}")),
        Ok(_) => Err("the broken vault loaded".to_owned()),
    }
}

fn error(path: &str, location: Location, line: usize, message: &str) -> NoteError {
    NoteError {
        path: PathBuf::from(path),
        problem: Problem {
            location,
            line: Some(line),
            message: message.to_owned(),
        },
    }
}

fn field(path: &str) -> Location {
    Location::Field(path.to_owned())
}

#[test]
fn the_fixture_vault_loads_completely() {
    let vault = Vault::load(&fixture_vault()).expect("the fixture vault loads");

    insta::assert_debug_snapshot!(vault.career());
}

#[test]
fn only_the_note_folders_and_the_profile_are_read() {
    let vault = Vault::load(&fixture_vault()).expect("the fixture vault loads");

    let notes: Vec<String> = vault
        .notes()
        .iter()
        .map(|note| format!("{} ({})", note.path.display(), note.note_type.slug()))
        .collect();

    insta::assert_debug_snapshot!(notes);
}

#[test]
fn a_note_keeps_its_text_and_body_sections() {
    let vault = Vault::load(&fixture_vault()).expect("the fixture vault loads");
    let profile = vault
        .notes()
        .iter()
        .find(|note| note.note_type == NoteType::Profile)
        .expect("the fixture vault has a profile");

    let titles: Vec<&str> = profile
        .body
        .sections
        .iter()
        .map(|section| section.title.as_str())
        .collect();
    let german = profile
        .body
        .section("Deutsch")
        .expect("the profile is translated");

    assert_eq!(vault.root(), fixture_vault());
    assert_eq!(
        profile.text,
        fs::read_to_string(fixture_vault().join("Profile.md")).unwrap()
    );
    assert_eq!(
        titles,
        ["Summary", "Strengths", "Self-PR", "日本語", "Deutsch"]
    );
    assert_eq!(
        german
            .subsection("Self-PR")
            .map(|subsection| subsection.lines.len()),
        Some(2)
    );
}

#[test]
fn a_link_written_in_nfd_resolves_to_the_nfc_name() {
    let vault = Vault::load(&fixture_vault()).expect("the fixture vault loads");
    let engagement = vault
        .career()
        .engagements
        .get(&"2019-04 Müller Logistik".parse().unwrap())
        .expect("the employment Engagement is read");

    let employer = engagement
        .employer
        .as_ref()
        .expect("an employment has an employer");

    assert_eq!(
        employer,
        &Link::<Organization>::new("Müller Logistik".parse().unwrap())
    );
    assert!(vault.career().get(employer).is_some());
}

#[test]
fn a_broken_fixture_names_file_line_and_field() {
    let vault = copy_of_fixture_vault("broken_field").unwrap();
    replace_in(
        &vault,
        "Projects/Acme - Shop.md",
        "team_size: 5",
        "team_size: five",
    )
    .unwrap();
    replace_in(
        &vault,
        "Skills/Go.md",
        "category: language",
        "category: lang",
    )
    .unwrap();

    assert_eq!(
        errors_of(&vault).unwrap(),
        [
            error(
                "Projects/Acme - Shop.md",
                field("team_size"),
                23,
                "must be a whole number from 1 to 4294967295"
            ),
            error(
                "Skills/Go.md",
                field("category"),
                6,
                "must be one of language, framework, library, database, cloud, platform, tool, ai-tool, \
                 method, human-language"
            ),
        ]
    );
}

#[test]
fn a_broken_link_names_the_missing_note() {
    let vault = copy_of_fixture_vault("broken_link").unwrap();
    fs::remove_file(vault.join("Skills/Docker.md")).unwrap();

    assert_eq!(
        errors_of(&vault).unwrap(),
        [error(
            "Projects/Acme - Shop.md",
            field("skills[2]"),
            36,
            "links to [[Docker]], but no note has that name"
        )]
    );
}

#[test]
fn two_notes_with_one_name_are_reported() {
    let vault = copy_of_fixture_vault("same_name").unwrap();
    fs::copy(
        vault.join("Skills/Go.md"),
        vault.join("Qualifications/Go.md"),
    )
    .unwrap();

    let errors = errors_of(&vault).unwrap();

    assert_eq!(
        errors,
        [NoteError {
            path: PathBuf::from("Skills/Go.md"),
            problem: Problem {
                location: Location::Note,
                line: None,
                message: "has the same name as Qualifications/Go.md".to_owned(),
            },
        }]
    );
}

#[test]
fn a_conditional_field_follows_the_field_it_depends_on() {
    let vault = copy_of_fixture_vault("conditional_field").unwrap();
    let note = "Engagements/2021-04 Acme.md";
    replace_in(
        &vault,
        note,
        "engagement_type: freelance",
        "engagement_type: own-business",
    )
    .unwrap();

    assert_eq!(
        errors_of(&vault).unwrap(),
        [error(
            note,
            field("contracts"),
            11,
            "must be empty because engagement_type is own-business"
        )]
    );
}

#[test]
fn a_missing_body_section_is_reported() {
    let vault = copy_of_fixture_vault("missing_section").unwrap();
    replace_in(
        &vault,
        "Monthly Logs/2022/2022-05 Acme Log.md",
        "## Focus\n",
        "## Topic\n",
    )
    .unwrap();

    assert_eq!(
        errors_of(&vault).unwrap(),
        [NoteError {
            path: PathBuf::from("Monthly Logs/2022/2022-05 Acme Log.md"),
            problem: Problem {
                location: Location::Note,
                line: None,
                message: "must have the sections `## Focus`, `## Outcomes` in this order, not \
                          `## Topic`, `## Outcomes`"
                    .to_owned(),
            },
        }]
    );
}

#[test]
fn a_missing_vault_cannot_be_read() {
    let missing = Path::new(env!("CARGO_TARGET_TMPDIR")).join("no-such-vault");

    let error = Vault::load(&missing).unwrap_err();

    assert!(
        matches!(error, LoadError::Unreadable { ref path, .. } if *path == missing),
        "{error}"
    );
}
