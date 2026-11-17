use crate::fixed_list::labelled_list;
use crate::{Date, Link, Month, Organization, Source, Yen};

/// Schema §2.3. `rate` is set exactly for `quasi-mandate`; `rate_unit` and `rate_history` only with a `rate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contract {
    pub counterparty: Link<Organization>,
    #[expect(
        clippy::struct_field_names,
        reason = "the fields carry the schema's key names (§2.3)"
    )]
    pub contract_type: ContractType,
    pub start: Date,
    /// Empty when the Contract is open-ended.
    pub end: Option<Date>,
    /// The current rate (schema §1.10).
    pub rate: Option<Yen>,
    pub rate_unit: Option<RateUnit>,
    pub rate_history: Vec<RateStep>,
    pub sources: Vec<Source>,
}

/// A rate that applies from `from` until the month before the next step (schema §1.10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateStep {
    pub from: Month,
    pub rate: Yen,
}

labelled_list!(pub ContractType {
    QuasiMandate => "quasi-mandate" [
        "Quasi-mandate contract (業務委託)", "業務委託契約", "Dienstleistungsvertrag (業務委託)"
    ],
    Employment => "employment" ["Employment contract", "雇用契約", "Arbeitsvertrag"],
    Dispatch => "dispatch" ["Dispatch employment contract", "派遣雇用契約", "Arbeitnehmerüberlassungsvertrag"],
    Internship => "internship" ["Internship agreement", "インターン契約", "Praktikumsvertrag"],
});

labelled_list!(pub RateUnit {
    Hour => "hour" ["Hourly rate", "時給", "Stundensatz"],
    Day => "day" ["Daily rate", "日給", "Tagessatz"],
    Month => "month" ["Monthly rate", "月額", "Monatssatz"],
});
