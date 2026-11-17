//! Schema §1.7: a Source cites a file that exists, and never a raw tracker export.

use std::fs;
use std::path::Path;

use kariera_vault::nfc;

use crate::context::Context;
use crate::field_path::FieldPath;
use crate::finding::{At, Finding, Rule};
use crate::sourced::{SourceOf, all_sources};

/// Raw tracker exports are never cited (schema §1.7); their digests under `sagyo/history/` are.
const RAW_EXPORTS: &str = "sagyo/history/raw/";

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    all_sources(context)
        .iter()
        .filter_map(|cited| path_problem(context.root(), cited))
        .collect()
}

fn path_problem(root: &Path, cited: &SourceOf<'_>) -> Option<Finding> {
    let path = cited.source.path.as_ref()?.as_str();
    let message = if nfc(path).starts_with(RAW_EXPORTS) {
        format!("must not point into `{RAW_EXPORTS}`: raw tracker exports are never cited")
    } else if file_exists(root, path) {
        return None;
    } else {
        format!("names {path:?}, which is not a file in the vault")
    };
    Some(Finding::new(
        cited.file,
        Rule::Source,
        At::Field(FieldPath::of("sources").item(cited.index).key("path")),
        message,
    ))
}

/// A file exists when each segment of its path names an entry of the directory above it, compared in NFC: macOS
/// and Syncthing write the same name in NFC and in NFD.
fn file_exists(root: &Path, relative: &str) -> bool {
    let relative = nfc(relative);
    let mut directory = root.to_path_buf();
    let mut segments = relative.split('/').peekable();
    while let Some(segment) = segments.next() {
        let Some(entry) = entry_named(&directory, segment) else {
            return false;
        };
        if segments.peek().is_none() {
            return entry.is_file();
        }
        directory = entry;
    }
    false
}

fn entry_named(directory: &Path, segment: &str) -> Option<std::path::PathBuf> {
    fs::read_dir(directory)
        .ok()?
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| nfc(name) == segment)
        })
        .map(|entry| entry.path())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::file_exists;
    use crate::test_vault::TestVault;

    const SKILL_WITH_SOURCE: &str = "Skills/TypeScript.md";
    const CITED: &str = ".scratch/_inputs/applications/skill-sheet-2023.xlsx";

    #[test]
    fn a_source_that_names_no_file_is_reported() {
        let vault = TestVault::new("sources_missing");
        vault.replace(
            SKILL_WITH_SOURCE,
            CITED,
            ".scratch/_inputs/applications/missing.xlsx",
        );

        assert_eq!(
            vault.lines(),
            [
                "Skills/TypeScript.md:13: error [source] field `sources[0].path` names \".scratch/_inputs/applications/missing.xlsx\", which is not a file in the vault"
            ]
        );
    }

    #[test]
    fn a_source_that_names_a_folder_is_reported() {
        let vault = TestVault::new("sources_folder");
        vault.replace(SKILL_WITH_SOURCE, CITED, ".scratch/_inputs/applications");

        assert_eq!(
            vault.lines(),
            [
                "Skills/TypeScript.md:13: error [source] field `sources[0].path` names \".scratch/_inputs/applications\", which is not a file in the vault"
            ]
        );
    }

    #[test]
    fn a_source_never_points_into_the_raw_tracker_exports() {
        let vault = TestVault::new("sources_raw");
        vault.write("sagyo/history/raw/backlog.json", "{}").replace(
            SKILL_WITH_SOURCE,
            CITED,
            "sagyo/history/raw/backlog.json",
        );

        assert_eq!(
            vault.lines(),
            [
                "Skills/TypeScript.md:13: error [source] field `sources[0].path` must not point into `sagyo/history/raw/`: raw tracker exports are never cited"
            ]
        );
    }

    #[test]
    fn a_digest_next_to_the_raw_exports_may_be_cited() {
        let vault = TestVault::new("sources_digest");
        vault
            .write("sagyo/history/digest.md", "digest")
            .write("sagyo/history/rawer/note.md", "x")
            .replace(SKILL_WITH_SOURCE, CITED, "sagyo/history/digest.md");

        assert_eq!(vault.lines(), Vec::<String>::new());
    }

    #[test]
    fn a_path_is_compared_in_nfc_against_a_file_written_in_nfd() {
        let vault = TestVault::new("sources_nfd");
        fs::create_dir_all(vault.root().join("inputs")).unwrap();
        fs::write(vault.root().join("inputs/Mu\u{308}ller.pdf"), "").unwrap();

        assert!(file_exists(vault.root(), "inputs/Müller.pdf"));
        assert!(file_exists(vault.root(), "inputs/Mu\u{308}ller.pdf"));
    }

    #[test]
    fn a_folder_written_in_nfd_is_found_too() {
        let vault = TestVault::new("sources_nfd_folder");
        fs::create_dir_all(vault.root().join("Mu\u{308}ller")).unwrap();
        fs::write(vault.root().join("Mu\u{308}ller/a.pdf"), "").unwrap();

        assert!(file_exists(vault.root(), "Müller/a.pdf"));
    }

    #[test]
    fn walking_finds_only_files_that_exist() {
        let vault = TestVault::new("sources_walk");

        assert!(file_exists(vault.root(), CITED));
        assert!(!file_exists(vault.root(), ".scratch/_inputs/applications"));
        assert!(!file_exists(
            vault.root(),
            ".scratch/_inputs/missing/skill-sheet-2023.xlsx"
        ));
        assert!(!file_exists(
            vault.root(),
            ".scratch/_inputs/applications/skill-sheet-2023.xlsx/more"
        ));
        assert!(!file_exists(
            Path::new("/nonexistent-kariera-directory"),
            "a.pdf"
        ));
    }
}
