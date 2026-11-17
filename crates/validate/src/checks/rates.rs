//! Schema §1.10: `rate_history` is in order, inside the Contract's period, and ends at `rate`.

use kariera_domain::{Contract, RateStep};

use crate::context::{Context, Found};
use crate::field_path::FieldPath;
use crate::finding::{At, Finding, Rule};
use crate::period::Period;

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    context
        .notes::<Contract>()
        .flat_map(|contract| {
            let mut findings = steps_in_order(&contract);
            findings.extend(steps_inside_the_period(&contract));
            findings.extend(last_step_is_the_rate(&contract));
            findings
        })
        .collect()
}

fn step_field(index: usize, key: &str) -> At {
    At::Field(FieldPath::of("rate_history").item(index).key(key))
}

fn steps_in_order(contract: &Found<'_, Contract>) -> Vec<Finding> {
    contract
        .note
        .rate_history
        .windows(2)
        .enumerate()
        .filter(|(_, pair)| matches!(pair, [earlier, later] if later.from <= earlier.from))
        .map(|(index, _)| {
            Finding::new(
                contract.file,
                Rule::Rate,
                step_field(index + 1, "from"),
                "must be later than the `from` of the entry before it: entries are in chronological order",
            )
        })
        .collect()
}

fn steps_inside_the_period(contract: &Found<'_, Contract>) -> Vec<Finding> {
    let period = Period::of_contract(contract.note);
    contract
        .note
        .rate_history
        .iter()
        .enumerate()
        .filter_map(|(index, step)| {
            period.explain_why_outside(step.from).map(|why| {
                Finding::new(
                    contract.file,
                    Rule::Rate,
                    step_field(index, "from"),
                    format!("{} {why} the Contract", step.from),
                )
            })
        })
        .collect()
}

fn last_step_is_the_rate(contract: &Found<'_, Contract>) -> Option<Finding> {
    let last: &RateStep = contract.note.rate_history.last()?;
    let rate = contract.note.rate?;
    (last.rate != rate).then(|| {
        Finding::new(
            contract.file,
            Rule::Rate,
            At::Field(FieldPath::of("rate")),
            format!(
                "is {rate:?}, but the last entry of `rate_history` is {:?}: `rate` is the current rate",
                last.rate
            ),
        )
    })
}

#[cfg(test)]
mod tests {
    use crate::test_vault::TestVault;

    const ACME_CONTRACT: &str = "Contracts/2021-04 Acme Contract.md";

    #[test]
    fn rate_history_entries_are_in_chronological_order() {
        let vault = TestVault::new("rates_order");
        vault.replace(ACME_CONTRACT, "  - from: 2022-04", "  - from: 2021-04");

        assert_eq!(
            vault.lines(),
            [
                "Contracts/2021-04 Acme Contract.md:12: error [rate] field `rate_history[1].from` must be later than the `from` of the entry before it: entries are in chronological order"
            ]
        );
    }

    #[test]
    fn rate_history_entries_may_not_run_backwards() {
        let vault = TestVault::new("rates_backwards");
        vault.replace(ACME_CONTRACT, "  - from: 2022-04", "  - from: 2021-01");

        assert_eq!(
            vault.lines(),
            [
                "Contracts/2021-04 Acme Contract.md:12: error [rate] field `rate_history[1].from` 2021-01 is before the start (2021-04) of the Contract",
                "Contracts/2021-04 Acme Contract.md:12: error [rate] field `rate_history[1].from` must be later than the `from` of the entry before it: entries are in chronological order",
            ]
        );
    }

    #[test]
    fn a_rate_step_lies_within_the_contract() {
        let vault = TestVault::new("rates_period");
        vault.replace(ACME_CONTRACT, "  - from: 2022-04", "  - from: 2023-10");

        assert_eq!(
            vault.lines(),
            [
                "Contracts/2021-04 Acme Contract.md:12: error [rate] field `rate_history[1].from` 2023-10 is after the end (2023-09) of the Contract"
            ]
        );
    }

    #[test]
    fn the_last_rate_step_is_the_current_rate() {
        let vault = TestVault::new("rates_last");
        vault.replace(
            ACME_CONTRACT,
            "rate: 5000\nrate_unit: hour",
            "rate: 5500\nrate_unit: hour",
        );

        assert_eq!(
            vault.lines(),
            [
                "Contracts/2021-04 Acme Contract.md:7: error [rate] field `rate` is 5500, but the last entry of `rate_history` is 5000: `rate` is the current rate"
            ]
        );
    }

    #[test]
    fn an_empty_rate_history_asks_for_nothing() {
        let vault = TestVault::new("rates_none");
        vault.replace(
            ACME_CONTRACT,
            "rate_history:\n  - from: 2021-04\n    rate: 4500\n  - from: 2022-04\n    rate: 5000\n",
            "",
        );

        assert_eq!(vault.lines(), Vec::<String>::new());
    }
}
