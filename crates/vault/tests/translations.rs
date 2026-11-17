use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use kariera_domain::TranslationLanguage;
use kariera_vault::{MarkError, Translation, Vault};

#[path = "support/vault_copy.rs"]
mod vault_copy;

use vault_copy::{copy_of_fixture_vault, fixture_vault};

const JAPANESE: TranslationLanguage = TranslationLanguage::Japanese;
const GERMAN: TranslationLanguage = TranslationLanguage::German;

const SHOP: &str = "Projects/Acme - Shop.md";

fn load(vault: &Path) -> Result<Vault, String> {
    Vault::load(vault).map_err(|error| error.to_string())
}

fn translation(
    vault: &Vault,
    name: &str,
    language: TranslationLanguage,
) -> Result<Translation, String> {
    vault
        .translations()
        .into_iter()
        .find(|translation| translation.name.as_str() == name && translation.language == language)
        .ok_or_else(|| format!("{name} has no translations"))
}

fn is_stale(vault: &Path, name: &str, language: TranslationLanguage) -> Result<bool, String> {
    Ok(translation(&load(vault)?, name, language)?.is_stale())
}

fn mark(vault: &Path, name: &str, language: TranslationLanguage) -> Result<(), String> {
    load(vault)?
        .mark(name, language)
        .map(drop)
        .map_err(|error| error.to_string())
}

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

/// Every file of a vault with its bytes.
fn files_of(vault: &Path) -> io::Result<Vec<(PathBuf, Vec<u8>)>> {
    let mut files = Vec::new();
    collect_files(vault, vault, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files(
    root: &Path,
    folder: &Path,
    files: &mut Vec<(PathBuf, Vec<u8>)>,
) -> io::Result<()> {
    for entry in fs::read_dir(folder)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(root, &path, files)?;
        } else {
            let relative = path.strip_prefix(root).map_err(io::Error::other)?;
            files.push((relative.to_owned(), fs::read(&path)?));
        }
    }
    Ok(())
}

#[test]
fn every_translatable_note_is_listed_with_both_languages() {
    let vault = load(&fixture_vault()).unwrap();

    let lines: Vec<String> = vault
        .translations()
        .iter()
        .map(|translation| {
            format!(
                "{} {} stored {} current {}",
                translation.path.display(),
                translation.language.code(),
                translation
                    .stored
                    .as_ref()
                    .map_or_else(|| "-".to_owned(), ToString::to_string),
                translation.current
            )
        })
        .collect();

    insta::assert_snapshot!(lines.join("\n"));
}

#[test]
fn contracts_and_monthly_logs_have_no_translations() {
    let vault = load(&fixture_vault()).unwrap();

    let listed: BTreeSet<_> = vault
        .translations()
        .into_iter()
        .map(|translation| translation.path)
        .collect();

    assert!(!listed.iter().any(|path| path.starts_with("Contracts")));
    assert!(!listed.iter().any(|path| path.starts_with("Monthly Logs")));
    assert!(listed.contains(Path::new(SHOP)));
}

#[test]
fn a_marked_translation_is_current_until_the_english_changes() {
    let vault = copy_of_fixture_vault("marked_until_english_changes").unwrap();
    assert!(is_stale(&vault, "Acme - Shop", JAPANESE).unwrap());

    mark(&vault, "Acme - Shop", JAPANESE).unwrap();

    assert!(!is_stale(&vault, "Acme - Shop", JAPANESE).unwrap());
    assert!(is_stale(&vault, "Acme - Shop", GERMAN).unwrap());
}

#[test]
fn editing_any_english_text_makes_the_translations_stale() {
    let edits = [
        (
            "title",
            "en: Contoso toy shop backend",
            "en: Contoso shop backend",
        ),
        ("role", "en: Backend developer", "en: Backend engineer"),
        ("focus", "en: Backend\n", "en: Server side\n"),
        ("industry", "en: E-commerce", "en: Retail"),
        ("description", "online toy shop", "online games shop"),
        (
            "second paragraph",
            "run in Docker containers",
            "run in containers",
        ),
        (
            "achievements",
            "- Cut the catalogue page load time by half",
            "- Halved the catalogue load time",
        ),
    ];
    for (part, from, to) in edits {
        let vault =
            copy_of_fixture_vault(&format!("english_edit_{}", part.replace(' ', "_"))).unwrap();
        mark(&vault, "Acme - Shop", JAPANESE).unwrap();
        mark(&vault, "Acme - Shop", GERMAN).unwrap();

        replace_in(&vault, SHOP, from, to).unwrap_or_else(|error| panic!("{part}: {error}"));

        assert!(is_stale(&vault, "Acme - Shop", JAPANESE).unwrap(), "{part}");
        assert!(is_stale(&vault, "Acme - Shop", GERMAN).unwrap(), "{part}");
    }
}

#[test]
fn editing_only_a_translation_changes_nothing() {
    let vault = copy_of_fixture_vault("translation_edit").unwrap();
    mark(&vault, "Acme - Shop", JAPANESE).unwrap();
    mark(&vault, "Acme - Shop", GERMAN).unwrap();

    replace_in(
        &vault,
        SHOP,
        "role:\n  en: Backend developer\n  ja: バックエンド開発者\n  de: Backend-Entwickler",
        "role:\n  en: Backend developer\n  ja: バックエンド技術者\n  de: Server-Entwickler",
    )
    .unwrap();
    replace_in(
        &vault,
        SHOP,
        "玩具ECサイトのバックエンドを",
        "おもちゃECのバックエンドを",
    )
    .unwrap();
    replace_in(
        &vault,
        SHOP,
        "- Entwarf den Bestell- und Zahlungsablauf",
        "- Gestaltete den Bestellablauf",
    )
    .unwrap();

    assert!(!is_stale(&vault, "Acme - Shop", JAPANESE).unwrap());
    assert!(!is_stale(&vault, "Acme - Shop", GERMAN).unwrap());
}

#[test]
fn whitespace_at_the_end_of_a_line_does_not_make_a_note_stale() {
    let vault = copy_of_fixture_vault("trailing_whitespace").unwrap();
    mark(&vault, "Acme - Shop", JAPANESE).unwrap();

    replace_in(&vault, SHOP, "Built the backend of an online toy shop in Go on PostgreSQL, from the product catalogue to order processing.\n", "Built the backend of an online toy shop in Go on PostgreSQL, from the product catalogue to order processing.  \n").unwrap();

    assert!(!is_stale(&vault, "Acme - Shop", JAPANESE).unwrap());
}

#[test]
fn marking_changes_only_the_marked_value() {
    let vault = copy_of_fixture_vault("mark_changes_one_value").unwrap();
    let before = files_of(&vault).unwrap();
    let note_before = fs::read_to_string(vault.join(SHOP)).unwrap();

    let fingerprint = load(&vault).unwrap().mark("Acme - Shop", JAPANESE).unwrap();

    let note_after = fs::read_to_string(vault.join(SHOP)).unwrap();
    assert_eq!(
        note_after,
        note_before.replacen("  ja: 1a2b3c4d5e6f\n", &format!("  ja: {fingerprint}\n"), 1)
    );
    let changed: Vec<_> = before
        .iter()
        .zip(files_of(&vault).unwrap())
        .filter(|(old, new)| *old != new)
        .map(|(old, _)| old.0.clone())
        .collect();
    assert_eq!(changed, [PathBuf::from(SHOP)]);
}

#[test]
fn marking_twice_gives_the_same_bytes() {
    let vault = copy_of_fixture_vault("mark_twice").unwrap();
    mark(&vault, "Acme - Shop", GERMAN).unwrap();
    let once = fs::read(vault.join(SHOP)).unwrap();

    mark(&vault, "Acme - Shop", GERMAN).unwrap();

    assert_eq!(fs::read(vault.join(SHOP)).unwrap(), once);
}

#[test]
fn a_note_without_translated_from_gets_it_before_the_next_field() {
    let vault = copy_of_fixture_vault("mark_adds_block").unwrap();
    let before = fs::read_to_string(vault.join("Skills/C Sharp.md")).unwrap();

    let fingerprint = load(&vault).unwrap().mark("C Sharp", GERMAN).unwrap();

    assert_eq!(
        fs::read_to_string(vault.join("Skills/C Sharp.md")).unwrap(),
        before.replacen(
            "reviewed: false\n",
            &format!("translated_from:\n  de: {fingerprint}\nreviewed: false\n"),
            1
        )
    );
    assert!(!is_stale(&vault, "C Sharp", GERMAN).unwrap());
    assert!(is_stale(&vault, "C Sharp", JAPANESE).unwrap());
}

#[test]
fn a_note_is_found_by_its_name_in_nfd_too() {
    let vault = copy_of_fixture_vault("mark_nfd_name").unwrap();

    let result = load(&vault)
        .unwrap()
        .mark("Mu\u{308}ller Logistik", JAPANESE);

    assert!(result.is_ok(), "{result:?}");
    assert!(!is_stale(&vault, "Müller Logistik", JAPANESE).unwrap());
}

#[test]
fn a_note_that_does_not_exist_cannot_be_marked() {
    let vault = copy_of_fixture_vault("mark_missing_note").unwrap();

    let result = load(&vault).unwrap().mark("No Such Note", JAPANESE);

    assert!(matches!(result, Err(MarkError::NoSuchNote(name)) if name == "No Such Note"));
}

#[test]
fn a_note_type_without_translated_from_cannot_be_marked() {
    let vault = copy_of_fixture_vault("mark_contract").unwrap();
    let before = files_of(&vault).unwrap();

    let result = load(&vault)
        .unwrap()
        .mark("2021-04 Acme Contract", JAPANESE);

    assert!(
        matches!(result, Err(MarkError::NotTranslatable { .. })),
        "{result:?}"
    );
    assert_eq!(files_of(&vault).unwrap(), before);
}

#[test]
fn a_note_that_cannot_be_written_is_reported_and_not_created() {
    let vault = copy_of_fixture_vault("mark_unwritable").unwrap();
    let loaded = load(&vault).unwrap();
    fs::remove_file(vault.join(SHOP)).unwrap();

    let result = loaded.mark("Acme - Shop", JAPANESE);

    assert!(matches!(result, Err(MarkError::Write { .. })), "{result:?}");
    assert!(!vault.join(SHOP).exists());
}

#[cfg(unix)]
#[test]
fn marking_keeps_the_permissions_and_leaves_no_temporary_file() {
    use std::os::unix::fs::PermissionsExt;

    let vault = copy_of_fixture_vault("mark_permissions").unwrap();
    fs::set_permissions(vault.join(SHOP), fs::Permissions::from_mode(0o600)).unwrap();
    let files_before: Vec<_> = files_of(&vault)
        .unwrap()
        .into_iter()
        .map(|file| file.0)
        .collect();

    mark(&vault, "Acme - Shop", JAPANESE).unwrap();

    let mode = fs::metadata(vault.join(SHOP)).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
    let hidden_files: Vec<_> = fs::read_dir(vault.join("Projects"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .filter(|name| name.to_string_lossy().ends_with(".kariera-tmp"))
        .collect();
    assert!(hidden_files.is_empty(), "{hidden_files:?}");
    let files_after: Vec<_> = files_of(&vault)
        .unwrap()
        .into_iter()
        .map(|file| file.0)
        .collect();
    assert_eq!(files_after, files_before);
}
