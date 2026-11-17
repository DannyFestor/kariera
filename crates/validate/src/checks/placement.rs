//! Schema §1.1 and §2.0: a note lives in the folder of its type.

use kariera_domain::{FixedList, NoteType};
use std::path::PathBuf;

use kariera_vault::{NoteFile, Stray, monthly_log_folder, nfc, note_folder};

use crate::context::Context;
use crate::finding::{At, Finding, Rule};

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    let misplaced = context
        .files()
        .iter()
        .filter_map(|file| misplaced_note(context, file));
    let strays = context.strays().iter().map(stray);
    misplaced.chain(strays).collect()
}

fn misplaced_note(context: &Context<'_>, file: &NoteFile) -> Option<Finding> {
    let expected = expected_folder(context, file)?;
    let actual = nfc(&file.path.parent()?.to_string_lossy());
    (actual != expected).then(|| {
        Finding::new(
            file,
            Rule::Folder,
            At::Note,
            format!(
                "is in {}, but a {} note belongs in {}",
                describe(&actual),
                file.note_type.slug(),
                describe(&expected)
            ),
        )
    })
}

/// `None` for a Monthly Log that did not read, which the vault reader has reported.
fn expected_folder(context: &Context<'_>, file: &NoteFile) -> Option<String> {
    let folder = match file.note_type {
        NoteType::MonthlyLog => {
            monthly_log_folder(context.career().monthly_logs.get(&file.name)?.month)
        }
        NoteType::Organization
        | NoteType::Engagement
        | NoteType::Contract
        | NoteType::Project
        | NoteType::Skill
        | NoteType::Qualification
        | NoteType::Profile => PathBuf::from(note_folder(file.note_type)),
    };
    Some(folder.to_string_lossy().into_owned())
}

fn describe(folder: &str) -> String {
    if folder.is_empty() {
        "the vault root".to_owned()
    } else {
        format!("`{folder}/`")
    }
}

fn stray(stray: &Stray) -> Finding {
    let declared = &stray.declared_type;
    let message = NoteType::from_slug(declared).map_or_else(
        || format!("declares `type: {declared}`, which is no note type, and sits outside every note folder"),
        |note_type| {
            let folder = describe(note_folder(note_type));
            format!("declares `type: {declared}`, so it belongs in {folder}, not here")
        },
    );
    Finding::at_path(&stray.path, Rule::Folder, message)
}

#[cfg(test)]
mod tests {
    use crate::test_vault::TestVault;

    #[test]
    fn a_note_in_the_folder_of_another_type_is_reported() {
        let vault = TestVault::new("placement_other_type");
        vault.rename("Skills/Go.md", "Projects/Go.md");

        assert_eq!(
            vault.lines(),
            [
                "Projects/Go.md: error [folder] the note is in `Projects/`, but a skill note belongs in `Skills/`"
            ]
        );
    }

    #[test]
    fn a_note_in_a_subfolder_of_its_folder_is_reported() {
        let vault = TestVault::new("placement_subfolder");
        vault.rename("Skills/Go.md", "Skills/Languages/Go.md");

        assert_eq!(
            vault.lines(),
            [
                "Skills/Languages/Go.md: error [folder] the note is in `Skills/Languages/`, but a skill note belongs in `Skills/`"
            ]
        );
    }

    #[test]
    fn a_monthly_log_belongs_in_the_folder_of_its_year() {
        let vault = TestVault::new("placement_log_year");
        vault.rename(
            "Monthly Logs/2025/2025-07 Globex Log.md",
            "Monthly Logs/2024/2025-07 Globex Log.md",
        );

        assert_eq!(
            vault.lines(),
            [
                "Monthly Logs/2024/2025-07 Globex Log.md: error [folder] the note is in `Monthly Logs/2024/`, but a monthly-log note belongs in `Monthly Logs/2025/`"
            ]
        );
    }

    #[test]
    fn the_profile_belongs_in_the_vault_root() {
        let vault = TestVault::new("placement_profile");
        vault.rename("Profile.md", "Skills/Profile.md");

        assert_eq!(
            vault.lines(),
            [
                "Skills/Profile.md: error [folder] the note is in `Skills/`, but a profile note belongs in the vault root"
            ]
        );
    }

    #[test]
    fn a_note_outside_every_note_folder_is_reported_by_its_type() {
        let vault = TestVault::new("placement_stray");
        vault.write(
            "Notes/Rust.md",
            "---\ntype: skill\nname: Rust\ncategory: language\nreviewed: false\n---\n",
        );

        assert_eq!(
            vault.lines(),
            [
                "Notes/Rust.md: error [folder] the note declares `type: skill`, so it belongs in `Skills/`, not here"
            ]
        );
    }

    #[test]
    fn a_stray_in_the_vault_root_is_reported() {
        let vault = TestVault::new("placement_root_stray");
        vault.write("Rust.md", "---\ntype: profile\n---\n");

        assert_eq!(
            vault.lines(),
            [
                "Rust.md: error [folder] the note declares `type: profile`, so it belongs in the vault root, not here"
            ]
        );
    }

    #[test]
    fn a_stray_with_a_type_no_note_has_is_reported() {
        let vault = TestVault::new("placement_unknown_type");
        vault.write("Notes/Idea.md", "---\ntype: idea\n---\n");

        assert_eq!(
            vault.lines(),
            [
                "Notes/Idea.md: error [folder] the note declares `type: idea`, which is no note type, and sits outside every note folder"
            ]
        );
    }

    #[test]
    fn markdown_without_a_type_and_the_exempt_folders_are_not_notes() {
        let vault = TestVault::new("placement_exempt");
        let typed = "---\ntype: skill\n---\n";
        vault
            .write("AGENTS.md", "# Instructions\n")
            .write("GLOSSARY.md", "---\ntitle: Words\n---\n")
            .write("Todo.md", "- [ ] something\n")
            .write("Templates/project.md", typed)
            .write("Generated/cv.md", typed)
            .write("docs/schema.md", typed)
            .write(".scratch/note.md", typed)
            .write("Leads/.hidden.md", typed)
            .write("Leads/readme.txt", typed);

        assert_eq!(vault.lines(), Vec::<String>::new());
    }
}
