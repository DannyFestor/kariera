//! Schema §1.11: `project_hours` belongs to the log's Engagement and month, and adds up to `hours`.

use kariera_domain::{Hours, Link, MonthlyLog, Project, SourceKind};

use crate::context::{Context, Found};
use crate::field_path::FieldPath;
use crate::finding::{At, Finding, Rule};
use crate::period::Period;

const MISMATCH_PREFIX: &str = "Hours mismatch:";

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    context
        .notes::<MonthlyLog>()
        .flat_map(|log| {
            let mut findings = projects_of_the_log(context, &log);
            findings.extend(sum_matches_or_is_explained(&log));
            findings.extend(cites_the_source_of_hours(&log));
            findings
        })
        .collect()
}

fn projects_of_the_log(context: &Context<'_>, log: &Found<'_, MonthlyLog>) -> Vec<Finding> {
    log.note
        .project_hours
        .keys()
        .filter_map(|link| {
            let project = context.get(link)?;
            project_mismatch(log, link, &project).map(|message| {
                Finding::new(
                    log.file,
                    Rule::Hours,
                    At::Field(
                        FieldPath::of("project_hours").key(&format!("[[{}]]", link.target())),
                    ),
                    message,
                )
            })
        })
        .collect()
}

/// Each key links a Project of the log's Engagement whose period includes the log's month.
fn project_mismatch(
    log: &Found<'_, MonthlyLog>,
    link: &Link<Project>,
    project: &Found<'_, Project>,
) -> Option<String> {
    if project.note.engagement != log.note.engagement {
        return Some(format!(
            "links [[{}]], a Project of another Engagement than [[{}]]",
            link.target(),
            log.note.engagement.target()
        ));
    }
    let month = log.note.month;
    (!Period::of_project(project.note).contains(month))
        .then(|| format!("links [[{}]], which does not run in {month}", link.target()))
}

/// Without `hours` there is nothing to add up to: hours are never derived (schema §1.11).
fn sum_matches_or_is_explained(log: &Found<'_, MonthlyLog>) -> Option<Finding> {
    let hours = log.note.hours?;
    let sum: Hours = log.note.project_hours.values().copied().sum();
    let differs = !log.note.project_hours.is_empty() && sum != hours;
    (differs && !has_mismatch_note(log.note)).then(|| {
        Finding::new(
            log.file,
            Rule::Hours,
            At::Field(FieldPath::of("project_hours")),
            format!(
                "adds up to {sum} h, but `hours` is {hours}: a Source `note` starting with `{MISMATCH_PREFIX}` must explain the difference"
            ),
        )
        .warning()
    })
}

/// `hours` comes from an Invoice, a Timesheet or a Work Report, and that Source is cited (schema §1.11). Which of
/// them it came from is not recorded, so a log is only reported when it cites none of the three. A log without
/// `hours` has no such Source.
fn cites_the_source_of_hours(log: &Found<'_, MonthlyLog>) -> Option<Finding> {
    log.note.hours?;
    let cites_one = log.note.sources.iter().any(|source| {
        matches!(
            source.kind,
            SourceKind::Invoice | SourceKind::Timesheet | SourceKind::WorkReport
        )
    });
    (!cites_one).then(|| {
        Finding::new(
            log.file,
            Rule::Hours,
            At::Field(FieldPath::of("sources")),
            "cites no invoice, timesheet or work report, but `hours` comes from one of them",
        )
        .warning()
    })
}

/// The prefix and an explanation after it.
fn has_mismatch_note(log: &MonthlyLog) -> bool {
    log.sources
        .iter()
        .filter_map(|source| source.note.as_ref())
        .filter_map(|note| note.as_str().strip_prefix(MISMATCH_PREFIX))
        .any(|explanation| !explanation.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use crate::test_vault::TestVault;

    const LOG: &str = "Monthly Logs/2025/2025-07 Globex Log.md";
    const EXPLAINED: &str =
        "\"Hours mismatch: the timesheet leaves out 2.25 h of planning that the invoice bills\"";

    /// The log of July 2025 with its mismatch note replaced by `note`.
    fn vault_with_note(test: &str, note: &str) -> TestVault {
        let vault = TestVault::new(test);
        vault.replace(LOG, EXPLAINED, note);
        vault
    }

    #[test]
    fn project_hours_that_differ_without_an_explanation_are_a_warning() {
        let vault = vault_with_note("hours_mismatch", "split by project");

        assert_eq!(
            vault.lines(),
            [
                "Monthly Logs/2025/2025-07 Globex Log.md:7: warning [hours] field `project_hours` adds up to 150.25 h, but `hours` is 152.5: a Source `note` starting with `Hours mismatch:` must explain the difference"
            ]
        );
        assert_eq!((vault.report().errors(), vault.report().warnings()), (0, 1));
    }

    #[test]
    fn the_mismatch_prefix_needs_an_explanation_after_it() {
        let vault = vault_with_note("hours_bare_prefix", "\"Hours mismatch:\"");

        assert_eq!(vault.report().warnings(), 1);
    }

    #[test]
    fn the_prefix_is_case_sensitive() {
        let vault = vault_with_note("hours_prefix_case", "\"hours mismatch: lower case\"");

        assert_eq!(vault.report().warnings(), 1);
    }

    #[test]
    fn project_hours_that_add_up_need_no_explanation() {
        let vault = vault_with_note("hours_adds_up", "split by project");
        vault.replace(LOG, "hours: 152.5", "hours: 150.25");

        assert_eq!(vault.lines(), Vec::<String>::new());
    }

    #[test]
    fn a_difference_of_one_hundredth_of_an_hour_is_a_mismatch() {
        let vault = vault_with_note("hours_hundredth", "split by project");
        vault.replace(LOG, "hours: 152.5", "hours: 150.26");

        assert_eq!(vault.report().warnings(), 1);
    }

    #[test]
    fn a_log_that_cites_no_source_of_hours_is_a_warning() {
        let vault = TestVault::new("hours_source_cited");
        vault.replace(
            "Monthly Logs/2022/2022-05 Acme Log.md",
            "  - kind: invoice\n    path: .scratch/_inputs/invoices/2022/acme-2022-05.pdf\n    note: 150 h\n",
            "  - kind: recalled\n    note: about 150 h\n",
        );

        assert_eq!(
            vault.lines(),
            [
                "Monthly Logs/2022/2022-05 Acme Log.md:7: warning [hours] field `sources` cites no invoice, timesheet or work report, but `hours` comes from one of them"
            ]
        );
    }

    #[test]
    fn a_timesheet_or_a_work_report_is_a_source_of_hours_too() {
        for kind in ["timesheet", "work-report"] {
            let vault = TestVault::new("hours_source_kinds");
            vault.replace(
                "Monthly Logs/2022/2022-05 Acme Log.md",
                "  - kind: invoice\n",
                &format!("  - kind: {kind}\n"),
            );

            assert_eq!(vault.lines(), Vec::<String>::new(), "{kind}");
        }
    }

    #[test]
    fn a_log_without_hours_has_nothing_to_add_up_to_and_no_source_of_hours_to_cite() {
        let vault = TestVault::new("hours_empty");
        vault.replace(
            "Monthly Logs/2025/2025-06 Globex Log.md",
            "hours: 160\n",
            "",
        );
        vault.replace(
            "Monthly Logs/2025/2025-06 Globex Log.md",
            "  - kind: invoice\n    path: .scratch/_inputs/invoices/2025/globex-2025-06.pdf\n    note: 160 h\n",
            "  - kind: recalled\n    note: no document states the hours\n",
        );

        assert_eq!(vault.lines(), Vec::<String>::new());
    }

    #[test]
    fn project_hours_without_hours_are_not_a_mismatch() {
        let vault = TestVault::new("hours_empty_with_project_hours");
        vault.replace(
            "Monthly Logs/2025/2025-06 Globex Log.md",
            "hours: 160\n",
            "",
        );

        assert_eq!(vault.lines(), Vec::<String>::new());
    }

    #[test]
    fn a_project_of_another_engagement_is_reported() {
        let vault = TestVault::new("hours_other_engagement");
        vault.replace(
            LOG,
            "\"[[Globex - Admin]]\": 50.25",
            "\"[[Acme - Shop]]\": 50.25",
        );

        let lines = vault.lines();

        assert!(
            lines.contains(&"Monthly Logs/2025/2025-07 Globex Log.md:8: error [hours] field `project_hours.[[Acme - Shop]]` links [[Acme - Shop]], a Project of another Engagement than [[2023-10 Globex]]".to_owned()),
            "{lines:#?}"
        );
    }

    #[test]
    fn a_project_that_does_not_run_in_the_month_is_reported() {
        let vault = TestVault::new("hours_other_month");
        vault.replace(
            "Projects/Globex - Admin.md",
            "start: 2025-01",
            "start: 2025-07",
        );

        assert_eq!(
            vault.lines(),
            [
                "Monthly Logs/2025/2025-06 Globex Log.md:8: error [hours] field `project_hours.[[Globex - Admin]]` links [[Globex - Admin]], which does not run in 2025-06"
            ]
        );
    }

    #[test]
    fn a_log_without_project_hours_has_nothing_to_add_up() {
        let vault = TestVault::new("hours_none");
        vault.replace(
            "Monthly Logs/2025/2025-06 Globex Log.md",
            "project_hours:\n  \"[[Globex - Search]]\": 120\n  \"[[Globex - Admin]]\": 40\n",
            "",
        );

        assert_eq!(vault.lines(), Vec::<String>::new());
    }
}
