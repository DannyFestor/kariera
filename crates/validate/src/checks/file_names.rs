//! Schema §1.1 and §2: a note's file name follows the pattern of its type.

use kariera_domain::{
    Contract, Engagement, MonthlyLog, NoteName, Organization, Project, Qualification, Skill, Text,
};

use crate::context::{Context, Found, client_link};
use crate::finding::{At, Finding, Rule};

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(
        context
            .notes::<Organization>()
            .filter_map(named_after(|o: &Organization| &o.name)),
    );
    findings.extend(
        context
            .notes::<Skill>()
            .filter_map(named_after(|s: &Skill| &s.name)),
    );
    findings.extend(
        context
            .notes::<Qualification>()
            .filter_map(named_after(|q: &Qualification| &q.name)),
    );
    findings.extend(context.notes::<Engagement>().filter_map(|e| engagement(&e)));
    findings.extend(
        context
            .notes::<Contract>()
            .filter_map(|c| contract(context, &c)),
    );
    findings.extend(
        context
            .notes::<Project>()
            .filter_map(|p| project(context, &p)),
    );
    findings.extend(
        context
            .notes::<MonthlyLog>()
            .filter_map(|l| monthly_log(context, &l)),
    );
    findings
}

/// What the file name of a note must be, and where each part of it comes from.
struct Pattern {
    shape: Shape,
    parts: &'static str,
}

enum Shape {
    Exactly(String),
    /// A fixed beginning followed by a name the author chooses.
    StartsWith(String),
}

impl Pattern {
    const fn exactly(name: String, parts: &'static str) -> Self {
        Self {
            shape: Shape::Exactly(name),
            parts,
        }
    }

    fn accepts(&self, name: &str) -> bool {
        match &self.shape {
            Shape::Exactly(expected) => name == expected,
            Shape::StartsWith(prefix) => name.starts_with(prefix.as_str()),
        }
    }

    fn describe(&self) -> String {
        match &self.shape {
            Shape::Exactly(expected) => format!("`{expected}`"),
            Shape::StartsWith(prefix) => format!("`{prefix}<Project>`"),
        }
    }
}

fn violation<T>(note: &Found<'_, T>, pattern: &Pattern) -> Option<Finding> {
    (!pattern.accepts(note.name.as_str())).then(|| {
        Finding::new(
            note.file,
            Rule::FileName,
            At::Note,
            format!(
                "must be named {} ({}), not `{}`",
                pattern.describe(),
                pattern.parts,
                note.name
            ),
        )
    })
}

/// An Organization, Skill or Qualification is named after its `name` (schema §2.1, §2.6, §2.7).
fn named_after<'a, T>(name_of: fn(&T) -> &Text) -> impl Fn(Found<'a, T>) -> Option<Finding> {
    move |note| {
        let name = name_of(note.note).to_string();
        violation(&note, &Pattern::exactly(name, "its field `name`"))
    }
}

fn engagement(engagement: &Found<'_, Engagement>) -> Option<Finding> {
    let client = client_link(engagement.note)?.target();
    let name = format!("{} {client}", engagement.note.start);
    violation(
        engagement,
        &Pattern::exactly(name, "its start month and its Client"),
    )
}

fn contract(context: &Context<'_>, contract: &Found<'_, Contract>) -> Option<Finding> {
    let engagement = context.engagement_of(contract)?;
    let client = client_link(engagement.note)?.target();
    let via = via(client, contract.note.counterparty.target());
    let name = format!("{} {client}{via} Contract", contract.note.start.month());
    violation(
        contract,
        &Pattern::exactly(
            name,
            "its start month, its Client, ` via <counterparty>` unless the counterparty is the Client, and `Contract`",
        ),
    )
}

/// Present if and only if the counterparty is not the Client (schema §1.1).
fn via(client: &NoteName, counterparty: &NoteName) -> String {
    if client == counterparty {
        String::new()
    } else {
        format!(" via {counterparty}")
    }
}

fn project(context: &Context<'_>, project: &Found<'_, Project>) -> Option<Finding> {
    let engagement = context.get(&project.note.engagement)?;
    let client = client_link(engagement.note)?.target();
    let pattern = Pattern {
        shape: Shape::StartsWith(format!("{client} - ")),
        parts: "its Client, ` - `, and a short English name",
    };
    violation(project, &pattern)
}

fn monthly_log(context: &Context<'_>, log: &Found<'_, MonthlyLog>) -> Option<Finding> {
    let engagement = context.get(&log.note.engagement)?;
    let client = client_link(engagement.note)?.target();
    let name = format!("{} {client} Log", log.note.month);
    violation(
        log,
        &Pattern::exactly(name, "its month, its Client and `Log`"),
    )
}

#[cfg(test)]
mod tests {
    use crate::test_vault::TestVault;

    #[test]
    fn an_organization_is_named_after_its_name_field() {
        let vault = TestVault::new("names_organization");
        vault.replace(
            "Organizations/Globex.md",
            "name: Globex",
            "name: Globex Corporation",
        );

        assert_eq!(
            vault.lines(),
            [
                "Organizations/Globex.md: error [file-name] the note must be named `Globex Corporation` (its field `name`), not `Globex`"
            ]
        );
    }

    #[test]
    fn a_skill_and_a_qualification_are_named_after_their_name_field() {
        let vault = TestVault::new("names_skill_qualification");
        vault
            .replace("Skills/Go.md", "name: Go", "name: Golang")
            .replace(
                "Qualifications/Class B Driving Licence.md",
                "name: Class B Driving Licence",
                "name: Driving Licence",
            );

        assert_eq!(
            vault.lines(),
            [
                "Qualifications/Class B Driving Licence.md: error [file-name] the note must be named `Driving Licence` (its field `name`), not `Class B Driving Licence`",
                "Skills/Go.md: error [file-name] the note must be named `Golang` (its field `name`), not `Go`",
            ]
        );
    }

    #[test]
    fn an_engagement_is_named_by_its_start_month_and_client() {
        let vault = TestVault::new("names_engagement");
        vault.replace(
            "Engagements/2018-07 Initech.md",
            "start: 2018-07",
            "start: 2018-08",
        );

        assert_eq!(
            vault.lines(),
            [
                "Engagements/2018-07 Initech.md: error [file-name] the note must be named `2018-08 Initech` (its start month and its Client), not `2018-07 Initech`"
            ]
        );
    }

    #[test]
    fn a_contract_is_named_by_its_start_month_and_client() {
        let vault = TestVault::new("names_contract_start");
        vault.replace(
            "Contracts/2025-04 Globex Contract.md",
            "start: 2025-04-01",
            "start: 2025-05-01",
        );

        assert_eq!(
            vault.lines(),
            [
                "Contracts/2025-04 Globex Contract.md: error [file-name] the note must be named `2025-05 Globex Contract` (its start month, its Client, ` via <counterparty>` unless the counterparty is the Client, and `Contract`), not `2025-04 Globex Contract`"
            ]
        );
    }

    #[test]
    fn a_contract_names_its_counterparty_exactly_when_it_is_not_the_client() {
        let vault = TestVault::new("names_contract_via");
        vault
            .replace(
                "Contracts/2023-10 Globex via Fabrikam Contract.md",
                "counterparty: \"[[Fabrikam]]\"",
                "counterparty: \"[[Globex]]\"",
            )
            .replace(
                "Contracts/2021-04 Acme Contract.md",
                "counterparty: \"[[Acme]]\"",
                "counterparty: \"[[Fabrikam]]\"",
            );

        assert_eq!(
            vault.lines(),
            [
                "Contracts/2021-04 Acme Contract.md: error [file-name] the note must be named `2021-04 Acme via Fabrikam Contract` (its start month, its Client, ` via <counterparty>` unless the counterparty is the Client, and `Contract`), not `2021-04 Acme Contract`",
                "Contracts/2023-10 Globex via Fabrikam Contract.md: error [file-name] the note must be named `2023-10 Globex Contract` (its start month, its Client, ` via <counterparty>` unless the counterparty is the Client, and `Contract`), not `2023-10 Globex via Fabrikam Contract`",
            ]
        );
    }

    #[test]
    fn a_project_starts_its_name_with_its_client() {
        let vault = TestVault::new("names_project");
        vault.rename("Projects/Acme - Ledger.md", "Projects/Ledger.md");

        assert_eq!(
            vault.lines(),
            [
                "Projects/Ledger.md: error [file-name] the note must be named `Acme - <Project>` (its Client, ` - `, and a short English name), not `Ledger`"
            ]
        );
    }

    #[test]
    fn a_project_name_is_the_client_a_dash_and_a_name_even_when_the_client_changes() {
        let vault = TestVault::new("names_project_client");
        vault.replace(
            "Projects/Acme - Ledger.md",
            "engagement: \"[[2021-04 Acme]]\"",
            "engagement: \"[[2023-10 Globex]]\"",
        );

        assert_eq!(
            vault.lines(),
            [
                "Projects/Acme - Ledger.md: error [file-name] the note must be named `Globex - <Project>` (its Client, ` - `, and a short English name), not `Acme - Ledger`",
                "Projects/Acme - Ledger.md:14: error [period] field `start` 2022-10 is before the start (2023-10) of the Engagement [[2023-10 Globex]]",
                "Projects/Acme - Ledger.md:15: error [period] field `end` 2023-09 is before the start (2023-10) of the Engagement [[2023-10 Globex]]",
            ]
        );
    }

    #[test]
    fn a_monthly_log_is_named_by_its_month_and_client() {
        let vault = TestVault::new("names_log");
        vault.rename(
            "Monthly Logs/2025/2025-07 Globex Log.md",
            "Monthly Logs/2025/2025-07 Globex Report.md",
        );

        assert_eq!(
            vault.lines(),
            [
                "Monthly Logs/2025/2025-07 Globex Report.md: error [file-name] the note must be named `2025-07 Globex Log` (its month, its Client and `Log`), not `2025-07 Globex Report`"
            ]
        );
    }
}
