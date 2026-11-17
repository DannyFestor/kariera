//! Schema §1.3, §2.4, §2.5 and §3: a period ends after it starts, and sits inside the period of its Engagement.

use kariera_domain::{Contract, Engagement, Month, MonthlyLog, Project};

use crate::context::{Context, Found};
use crate::field_path::FieldPath;
use crate::finding::{At, Finding, Rule};
use crate::period::Period;

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(context.notes::<Engagement>().filter_map(|e| engagement(&e)));
    findings.extend(
        context
            .notes::<Project>()
            .flat_map(|p| project(context, &p)),
    );
    findings.extend(
        context
            .notes::<Contract>()
            .flat_map(|c| contract(context, &c)),
    );
    findings.extend(
        context
            .notes::<MonthlyLog>()
            .filter_map(|l| monthly_log(context, &l)),
    );
    findings
}

fn engagement(engagement: &Found<'_, Engagement>) -> Option<Finding> {
    ends_before_start(engagement, engagement.note.start, engagement.note.end)
}

fn project(context: &Context<'_>, project: &Found<'_, Project>) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(ends_before_start(
        project,
        project.note.start,
        project.note.end,
    ));
    if let Some(engagement) = context.get(&project.note.engagement) {
        findings.extend(outside(project, "start", project.note.start, &engagement));
        findings.extend(project.note.end.map_or_else(
            || ongoing_in_ended_engagement(project, &engagement),
            |end| outside(project, "end", end, &engagement),
        ));
    }
    findings
}

fn contract(context: &Context<'_>, contract: &Found<'_, Contract>) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(ends_before_start(
        contract,
        contract.note.start,
        contract.note.end,
    ));
    for engagement in context.engagements_listing(contract.name) {
        let period = Period::of_contract(contract.note);
        findings.extend(outside(contract, "start", period.start, &engagement));
        findings.extend(
            period
                .end
                .and_then(|end| outside(contract, "end", end, &engagement)),
        );
    }
    findings
}

fn monthly_log(context: &Context<'_>, log: &Found<'_, MonthlyLog>) -> Option<Finding> {
    let engagement = context.get(&log.note.engagement)?;
    outside(log, "month", log.note.month, &engagement)
}

/// A non-empty `end` must not precede `start` (schema §1.3).
fn ends_before_start<T, P: PartialOrd + Copy + std::fmt::Display>(
    note: &Found<'_, T>,
    start: P,
    end: Option<P>,
) -> Option<Finding> {
    end.filter(|end| *end < start).map(|_| {
        Finding::new(
            note.file,
            Rule::Period,
            At::Field(FieldPath::of("end")),
            format!("must not be before `start` ({start})"),
        )
    })
}

/// `month` of the field `field` must lie in the Engagement's period.
fn outside<T>(
    note: &Found<'_, T>,
    field: &str,
    month: Month,
    engagement: &Found<'_, Engagement>,
) -> Option<Finding> {
    Period::of_engagement(engagement.note)
        .explain_why_outside(month)
        .map(|why| {
            Finding::new(
                note.file,
                Rule::Period,
                At::Field(FieldPath::of(field)),
                format!("{month} {why} the Engagement [[{}]]", engagement.name),
            )
        })
}

/// A Project without an `end` is ongoing, which only an ongoing Engagement allows (schema §2.4).
fn ongoing_in_ended_engagement(
    project: &Found<'_, Project>,
    engagement: &Found<'_, Engagement>,
) -> Option<Finding> {
    engagement.note.end.map(|end| {
        Finding::new(
            project.file,
            Rule::Period,
            At::Field(FieldPath::of("end")),
            format!(
                "must be set: an empty `end` means ongoing, but the Engagement [[{}]] ended {end}",
                engagement.name
            ),
        )
    })
}

#[cfg(test)]
mod tests {
    use crate::test_vault::TestVault;

    #[test]
    fn an_engagement_must_not_end_before_it_starts() {
        let vault = TestVault::new("periods_engagement");
        vault.replace(
            "Engagements/2018-07 Initech.md",
            "end: 2018-09",
            "end: 2018-06",
        );

        assert_eq!(
            vault.lines(),
            [
                "Engagements/2018-07 Initech.md:6: error [period] field `end` must not be before `start` (2018-07)"
            ]
        );
    }

    #[test]
    fn a_project_must_not_end_before_it_starts() {
        let vault = TestVault::new("periods_project");
        vault.replace("Projects/Acme - Ledger.md", "end: 2023-09", "end: 2022-09");

        assert_eq!(
            vault.lines(),
            [
                "Projects/Acme - Ledger.md:15: error [period] field `end` must not be before `start` (2022-10)"
            ]
        );
    }

    #[test]
    fn a_contract_must_not_end_before_it_starts_by_the_day() {
        let vault = TestVault::new("periods_contract");
        vault.replace(
            "Contracts/2023-10 Globex via Fabrikam Contract.md",
            "end: 2025-03-31",
            "end: 2023-09-30",
        );

        assert_eq!(
            vault.lines(),
            [
                "Contracts/2023-10 Globex via Fabrikam Contract.md:6: error [period] field `end` 2023-09 is before the start (2023-10) of the Engagement [[2023-10 Globex]]",
                "Contracts/2023-10 Globex via Fabrikam Contract.md:6: error [period] field `end` must not be before `start` (2023-10-01)",
            ]
        );
    }

    #[test]
    fn a_contract_may_end_on_the_day_it_starts() {
        let vault = TestVault::new("periods_contract_same_day");
        vault.replace(
            "Contracts/2025-04 Globex Contract.md",
            "start: 2025-04-01",
            "start: 2025-04-01\nend: 2025-04-01",
        );

        assert_eq!(vault.lines(), Vec::<String>::new());
    }

    #[test]
    fn a_project_must_not_start_before_its_engagement() {
        let vault = TestVault::new("periods_project_start");
        vault.replace(
            "Projects/Acme - Shop.md",
            "start: 2021-04",
            "start: 2021-03",
        );

        assert_eq!(
            vault.lines(),
            [
                "Projects/Acme - Shop.md:13: error [period] field `start` 2021-03 is before the start (2021-04) of the Engagement [[2021-04 Acme]]"
            ]
        );
    }

    #[test]
    fn a_project_must_not_end_after_its_engagement() {
        let vault = TestVault::new("periods_project_end");
        vault.replace("Projects/Acme - Ledger.md", "end: 2023-09", "end: 2023-10");

        assert_eq!(
            vault.lines(),
            [
                "Projects/Acme - Ledger.md:15: error [period] field `end` 2023-10 is after the end (2023-09) of the Engagement [[2021-04 Acme]]"
            ]
        );
    }

    #[test]
    fn a_project_may_cover_its_whole_engagement() {
        let vault = TestVault::new("periods_project_whole");
        vault.replace(
            "Projects/Acme - Ledger.md",
            "start: 2022-10",
            "start: 2021-04",
        );

        assert_eq!(vault.lines(), Vec::<String>::new());
    }

    #[test]
    fn an_ongoing_project_needs_an_ongoing_engagement() {
        let vault = TestVault::new("periods_project_ongoing");
        vault.replace("Projects/Acme - Ledger.md", "end: 2023-09\n", "");

        assert_eq!(
            vault.lines(),
            [
                "Projects/Acme - Ledger.md:2: error [period] field `end` must be set: an empty `end` means ongoing, but the Engagement [[2021-04 Acme]] ended 2023-09"
            ]
        );
    }

    #[test]
    fn a_monthly_log_lies_within_its_engagement() {
        let vault = TestVault::new("periods_log");
        vault
            .replace(
                "Monthly Logs/2022/2022-05 Acme Log.md",
                "month: 2022-05",
                "month: 2023-10",
            )
            .rename(
                "Monthly Logs/2022/2022-05 Acme Log.md",
                "Monthly Logs/2023/2023-10 Acme Log.md",
            );

        assert_eq!(
            vault.lines(),
            [
                "Monthly Logs/2023/2023-10 Acme Log.md:4: error [period] field `month` 2023-10 is after the end (2023-09) of the Engagement [[2021-04 Acme]]"
            ]
        );
    }

    #[test]
    fn a_contract_starts_and_ends_within_its_engagement() {
        let vault = TestVault::new("periods_contract_inside");
        vault.replace(
            "Contracts/2021-04 Acme Contract.md",
            "end: 2023-09-30\nrate: 5000",
            "end: 2023-10-31\nrate: 5000",
        );

        assert_eq!(
            vault.lines(),
            [
                "Contracts/2021-04 Acme Contract.md:6: error [period] field `end` 2023-10 is after the end (2023-09) of the Engagement [[2021-04 Acme]]"
            ]
        );
    }

    #[test]
    fn a_contract_may_not_start_before_its_engagement() {
        let vault = TestVault::new("periods_contract_start");
        vault.replace(
            "Contracts/2025-04 Globex Contract.md",
            "start: 2025-04-01",
            "start: 2023-09-30",
        );

        assert_eq!(
            vault.lines(),
            [
                "Contracts/2025-04 Globex Contract.md: error [file-name] the note must be named `2023-09 Globex Contract` (its start month, its Client, ` via <counterparty>` unless the counterparty is the Client, and `Contract`), not `2025-04 Globex Contract`",
                "Contracts/2025-04 Globex Contract.md:5: error [period] field `start` 2023-09 is before the start (2023-10) of the Engagement [[2023-10 Globex]]",
            ]
        );
    }

    #[test]
    fn an_open_ended_contract_of_an_ended_engagement_is_not_reported() {
        let vault = TestVault::new("periods_contract_open");
        vault.replace(
            "Contracts/2021-04 Acme Contract.md",
            "end: 2023-09-30\n",
            "",
        );

        assert_eq!(vault.lines(), Vec::<String>::new());
    }
}
