use kariera_domain::{Contract, ContractType, FixedList};

use crate::fields::{Fields, Presence};
use crate::problems::Reported;
use crate::sections::Sections;

/// Schema §2.3 and §1.10.
pub(super) fn read(
    fields: &mut Fields<'_>,
    _empty_body: &mut Sections<'_>,
) -> Result<Contract, Reported> {
    let counterparty = fields.required("counterparty");
    let contract_type = fields.required::<ContractType>("contract_type");
    let start = fields.required("start");
    let end = fields.optional("end");
    let rate = fields.conditional("rate", contract_type.map(rate_presence));
    let has_rate = rate.map(|rate| rate.is_some());
    let rate_unit = fields.conditional("rate_unit", has_rate.map(rate_unit_presence));
    let rate_history = fields.conditional("rate_history", has_rate.map(rate_history_presence));
    let sources = fields.required("sources");
    Ok(Contract {
        counterparty: counterparty?,
        contract_type: contract_type?,
        start: start?,
        end: end?,
        rate: rate?,
        rate_unit: rate_unit?,
        rate_history: rate_history?.unwrap_or_default(),
        sources: sources?,
    })
}

fn rate_presence(contract_type: ContractType) -> Presence {
    let because = format!("contract_type is {}", contract_type.slug());
    if contract_type == ContractType::QuasiMandate {
        Presence::Required(because)
    } else {
        Presence::Empty(because)
    }
}

fn rate_unit_presence(has_rate: bool) -> Presence {
    if has_rate {
        Presence::Required("rate is set".to_owned())
    } else {
        Presence::Empty("rate is empty".to_owned())
    }
}

fn rate_history_presence(has_rate: bool) -> Presence {
    if has_rate {
        Presence::Optional
    } else {
        Presence::Empty("rate is empty".to_owned())
    }
}
