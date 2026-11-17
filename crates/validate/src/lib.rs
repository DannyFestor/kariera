//! Checks a vault against the schema and against itself (ADR 0001).
//!
//! The vault reader checks every note on its own, and its problems are findings of rule `schema`. The other rules
//! (file names, folders, field order and style, periods, links between notes, rates, hours, Sources, body content,
//! confidentiality) run over the notes that read. Only the rules that need every note, such as a Contract being
//! listed by exactly one Engagement, wait until every note reads.

mod body_lines;
mod checks;
mod context;
mod field_path;
mod finding;
mod layout;
mod period;
mod report;
mod sourced;
#[cfg(test)]
mod test_vault;

use std::path::Path;

use kariera_vault::{Inspection, LoadError, Vault};

pub use finding::{Finding, Rule, Severity};
pub use report::Report;

/// Checks the vault at `root`. Fails only when a file or folder cannot be read at all; a vault full of problems is
/// a `Report`.
pub fn validate(root: &Path) -> Result<Report, LoadError> {
    let Inspection { vault, errors } = Vault::inspect(root)?;
    let complete = errors.is_empty();
    let context = context::Context::new(&vault, complete)?;
    let mut findings: Vec<Finding> = errors.into_iter().map(Finding::from).collect();
    findings.extend(checks::run(&context));
    Ok(Report::new(findings, vault.notes().len(), complete))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use kariera_vault::LoadError;

    use crate::test_vault::TestVault;
    use crate::validate;

    #[test]
    fn the_fixture_vault_has_no_findings() {
        let vault = TestVault::new("the_fixture_vault_has_no_findings");

        let report = vault.report();

        assert_eq!(report.findings(), []);
        assert_eq!(report.notes_checked(), 43);
        assert!(report.is_complete());
        assert!(!report.has_errors());
    }

    #[test]
    fn a_vault_that_cannot_be_read_at_all_is_an_error_not_a_report() {
        let result = validate(Path::new("/nonexistent-kariera-vault"));

        assert!(
            matches!(result, Err(LoadError::Unreadable { .. })),
            "{result:?}"
        );
    }

    #[test]
    fn an_unknown_field_is_a_schema_error_of_its_note() {
        let vault = TestVault::new("reader_unknown_field");
        vault.replace(
            "Skills/Go.md",
            "category: language",
            "category: language\nlevel_of_love: high",
        );

        assert_eq!(
            vault.lines(),
            [
                "Skills/Go.md:7: error [schema] field `level_of_love` is not a field the schema allows here"
            ]
        );
    }

    #[test]
    fn a_missing_required_field_is_a_schema_error() {
        let vault = TestVault::new("reader_missing_field");
        vault.replace("Skills/Go.md", "category: language\n", "");

        assert_eq!(
            vault.lines(),
            ["Skills/Go.md:2: error [schema] field `category` is required"]
        );
    }

    #[test]
    fn a_value_in_the_wrong_format_is_a_schema_error() {
        let vault = TestVault::new("reader_wrong_format");
        vault.replace("Projects/Acme - Shop.md", "team_size: 5", "team_size: five");

        assert_eq!(
            vault.lines(),
            [
                "Projects/Acme - Shop.md:23: error [schema] field `team_size` must be a whole number from 1 to 4294967295"
            ]
        );
    }

    #[test]
    fn a_value_outside_a_fixed_list_is_a_schema_error() {
        let vault = TestVault::new("reader_fixed_list");
        vault.replace("Skills/Go.md", "category: language", "category: dialect");

        assert_eq!(
            vault.lines(),
            [
                "Skills/Go.md:6: error [schema] field `category` must be one of language, framework, library, database, cloud, platform, tool, ai-tool, method, human-language"
            ]
        );
    }

    #[test]
    fn a_link_to_no_note_is_a_schema_error() {
        let vault = TestVault::new("reader_broken_link");
        vault.replace(
            "Projects/Acme - Ledger.md",
            "  - \"[[PostgreSQL]]\"",
            "  - \"[[Postgres]]\"",
        );

        assert_eq!(
            vault.lines(),
            [
                "Projects/Acme - Ledger.md:24: error [schema] field `skills[1]` links to [[Postgres]], but no note has that name"
            ]
        );
    }

    #[test]
    fn a_link_to_a_skill_that_does_not_exist_is_a_schema_error() {
        let vault = TestVault::new("reader_missing_skill");
        vault.remove("Skills/Docker.md");

        assert_eq!(
            vault.lines(),
            [
                "Projects/Acme - Shop.md:36: error [schema] field `skills[2]` links to [[Docker]], but no note has that name"
            ]
        );
    }

    #[test]
    fn a_link_to_a_note_of_the_wrong_type_is_a_schema_error() {
        let vault = TestVault::new("reader_wrong_type");
        vault.replace(
            "Projects/Acme - Ledger.md",
            "  - \"[[PostgreSQL]]\"",
            "  - \"[[Acme]]\"",
        );

        assert_eq!(
            vault.lines(),
            [
                "Projects/Acme - Ledger.md:24: error [schema] field `skills[1]` links to [[Acme]], whose type is organization, not skill"
            ]
        );
    }

    #[test]
    fn a_note_without_sources_is_a_schema_error() {
        let vault = TestVault::new("reader_no_sources");
        vault.replace(
            "Projects/Globex - Admin.md",
            "sources:\n  - kind: timesheet\n    path: .scratch/_inputs/timesheets/globex-2025.xlsx\n",
            "",
        );

        assert_eq!(
            vault.lines(),
            ["Projects/Globex - Admin.md:2: error [schema] field `sources` is required"]
        );
    }

    #[test]
    fn a_source_with_a_file_needs_a_path_and_a_recalled_one_needs_a_note() {
        let vault = TestVault::new("reader_source_parts");
        vault
            .replace(
                "Engagements/2021-04 Acme.md",
                "    path: .scratch/_inputs/contracts/acme/basic-agreement.pdf\n",
                "",
            )
            .replace(
                "Projects/Acme - Shop.md",
                "  - kind: recalled\n    note: \"owner: the shop went live in 2021-11\"\n",
                "  - kind: recalled\n",
            );

        assert_eq!(
            vault.lines(),
            [
                "Engagements/2021-04 Acme.md:13: error [schema] field `sources[0].path` is required because kind is contract",
                "Projects/Acme - Shop.md:49: error [schema] field `sources[3].note` is required because kind is recalled",
            ]
        );
    }

    #[test]
    fn two_notes_of_one_name_are_a_schema_error_on_the_later_one() {
        let vault = TestVault::new("reader_duplicate_name");
        let go = "---\ntype: skill\nname: Go\ncategory: language\nreviewed: false\n---\n";
        vault.write("Skills/More/Go.md", go);

        assert_eq!(
            vault.lines(),
            ["Skills/More/Go.md: error [schema] the note has the same name as Skills/Go.md"]
        );
    }

    #[test]
    fn the_rules_about_a_single_note_run_on_the_notes_that_read() {
        let vault = TestVault::new("reader_partial");
        vault
            .replace("Skills/Go.md", "category: language", "category: dialect")
            .rename("Skills/Docker.md", "Projects/Docker.md");

        let report = vault.report();

        assert_eq!(
            vault.lines(),
            [
                "Projects/Docker.md: error [folder] the note is in `Projects/`, but a skill note belongs in `Skills/`",
                "Skills/Go.md:6: error [schema] field `category` must be one of language, framework, library, database, cloud, platform, tool, ai-tool, method, human-language",
            ]
        );
        assert!(!report.is_complete());
        assert_eq!(report.notes_checked(), 42);
    }

    #[test]
    fn the_rules_that_need_every_note_wait_until_every_note_reads() {
        let vault = TestVault::new("reader_incomplete");
        vault
            .replace("Skills/Go.md", "category: language", "category: dialect")
            .remove("Profile.md")
            .replace(
                "Engagements/2023-10 Globex.md",
                "  - \"[[2025-04 Globex Contract]]\"\n",
                "",
            );

        assert_eq!(
            vault.lines(),
            [
                "Skills/Go.md:6: error [schema] field `category` must be one of language, framework, library, database, cloud, platform, tool, ai-tool, method, human-language"
            ]
        );
    }

    #[test]
    fn the_findings_come_in_the_order_of_paths_and_lines() {
        let vault = TestVault::new("reader_order");
        vault
            .replace("Skills/Go.md", "category: language", "category: dialect")
            .replace(
                "Organizations/Acme.md",
                "  - client\n  - agency",
                "  - nonsense",
            )
            .replace("Projects/Acme - Shop.md", "team_size: 5", "team_size: five");

        let paths: Vec<String> = vault
            .report()
            .findings()
            .iter()
            .map(|finding| finding.path.display().to_string())
            .collect();

        assert_eq!(
            paths,
            [
                "Organizations/Acme.md",
                "Projects/Acme - Shop.md",
                "Skills/Go.md"
            ]
        );
    }

    #[test]
    fn warnings_do_not_make_a_vault_invalid() {
        let vault = TestVault::new("warnings_only");
        vault.replace(
            "Monthly Logs/2025/2025-07 Globex Log.md",
            "\"Hours mismatch: the timesheet leaves out 2.25 h of planning that the invoice bills\"",
            "split by project",
        );

        let report = vault.report();

        assert!(!report.has_errors());
        assert_eq!((report.errors(), report.warnings()), (0, 1));
    }

    #[test]
    fn errors_and_warnings_are_counted_apart() {
        let vault = TestVault::new("report_counts");
        let explained = "\"Hours mismatch: the timesheet leaves out 2.25 h of planning that the invoice bills\"";
        vault
            .replace(
                "Monthly Logs/2025/2025-07 Globex Log.md",
                explained,
                "split by project",
            )
            .replace(
                "Monthly Logs/2025/2025-06 Globex Log.md",
                "hours: 160",
                "hours: 161",
            )
            .replace(
                "Monthly Logs/2025/2025-06 Globex Log.md",
                "note: Sheet 2025-06, split by project",
                "note: split by project",
            );
        vault.replace(
            "Skills/Go.md",
            "aliases:\n  - Golang\n",
            "aliases: [Golang]\n",
        );

        let report = vault.report();

        assert_eq!((report.errors(), report.warnings()), (1, 2));
        assert!(report.has_errors());
    }
}
