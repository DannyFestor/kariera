use std::fs;
use std::path::{Path, PathBuf};

use kariera_domain::{FixedList, Link, NoteType, Organization};
use kariera_vault::{Inspection, LoadError, Location, NoteError, Problem, Stray, Value, Vault};

#[path = "support/vault_copy.rs"]
mod vault_copy;

#[path = "support/replace_in.rs"]
mod replace_in_support;

use replace_in_support::replace_in;
use vault_copy::{copy_of_fixture_vault, fixture_vault};

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

#[test]
fn a_note_keeps_its_frontmatter_with_the_style_of_every_value() {
    let vault = Vault::load(&fixture_vault()).expect("the fixture vault loads");
    let go = vault
        .notes()
        .iter()
        .find(|note| note.name.as_str() == "Go")
        .expect("the fixture vault has the Skill Go");

    let keys: Vec<&str> = match &go.frontmatter.value {
        Value::Mapping(entries) => entries.iter().map(|entry| entry.key.as_str()).collect(),
        Value::Scalar { .. } | Value::Sequence(_) => Vec::new(),
    };

    assert_eq!(keys, ["type", "name", "aliases", "category", "reviewed"]);
    assert_eq!(
        go.frontmatter
            .get("aliases")
            .map(|node| (node.line, node.flow)),
        Some((5, false))
    );
}

fn stray(path: &str, declared_type: &str) -> Stray {
    Stray {
        path: PathBuf::from(path),
        declared_type: declared_type.to_owned(),
    }
}

#[test]
fn the_fixture_vault_has_no_note_outside_its_folders() {
    let vault = Vault::load(&fixture_vault()).expect("the fixture vault loads");

    assert_eq!(vault.strays().expect("the vault is readable"), []);
}

#[test]
fn markdown_with_a_type_outside_the_note_folders_is_a_stray() {
    let copy = copy_of_fixture_vault("strays_with_a_type").expect("the copy is made");
    let typed = "---\ntype: skill\n---\n";
    for (path, text) in [
        ("Todo.md", "- [ ] write notes\n"),
        ("AGENTS.md", "# Instructions\n"),
        ("Leads/Idea.md", "---\ntype: idea\n---\n"),
        ("Leads/Deep/Rust.md", typed),
        ("Root.md", typed),
        ("Leads/Empty type.md", "---\ntype:\n---\n"),
        ("Leads/Broken.md", "---\ntype: [\n---\n"),
        ("Templates/skill-copy.md", typed),
        ("Generated/out.md", typed),
        ("docs/schema.md", typed),
        (".scratch/inbox.md", typed),
        (".obsidian/notes.md", typed),
        ("Leads/.hidden.md", typed),
        ("Leads/diagram.canvas", typed),
    ] {
        let target = copy.join(path);
        fs::create_dir_all(target.parent().expect("a file has a folder"))
            .expect("the folder is made");
        fs::write(target, text).expect("the file is written");
    }
    let vault = Vault::load(&copy).expect("the copy still loads");

    assert_eq!(
        vault.strays().expect("the vault is readable"),
        [
            stray("Leads/Deep/Rust.md", "skill"),
            stray("Leads/Idea.md", "idea"),
            stray("Root.md", "skill"),
        ]
    );
}

#[test]
fn inspecting_keeps_the_notes_that_read_and_the_problems_of_those_that_did_not() {
    let copy = copy_of_fixture_vault("inspect_partial").expect("the copy is made");
    replace_in(
        &copy,
        "Skills/Go.md",
        "category: language",
        "category: dialect",
    )
    .expect("the note is broken");

    let Inspection { vault, errors } = Vault::inspect(&copy).expect("the vault is readable");

    assert_eq!(vault.notes().len(), 42);
    assert!(vault.notes().iter().all(|note| note.name.as_str() != "Go"));
    assert_eq!(errors.len(), 1);
    assert!(
        matches!(Vault::load(&copy), Err(LoadError::InvalidNotes(load_errors)) if load_errors == errors)
    );
}
