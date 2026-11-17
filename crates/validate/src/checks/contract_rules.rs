//! Schema §2.3 and §3: a Contract belongs to exactly one Engagement, which fixes its type.

use kariera_domain::{Contract, ContractType, EngagementType, FixedList};

use crate::context::{Context, Found};
use crate::field_path::FieldPath;
use crate::finding::{At, Finding, Rule};

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    context
        .notes::<Contract>()
        .flat_map(|contract| {
            let listing = context.engagements_listing(contract.name);
            let mut findings = Vec::new();
            if context.is_complete() {
                findings.extend(listed_exactly_once(&contract, &listing));
            }
            findings.extend(
                listing
                    .iter()
                    .filter_map(|engagement| type_fixed_by(&contract, engagement)),
            );
            findings
        })
        .collect()
}

fn listed_exactly_once(
    contract: &Found<'_, Contract>,
    listing: &[Found<'_, kariera_domain::Engagement>],
) -> Option<Finding> {
    let message = match listing {
        [_] => return None,
        [] => {
            "is listed by no Engagement, but every Contract belongs to the Engagement that lists it"
                .to_owned()
        }
        several => format!(
            "is listed by {} Engagements ({}), but a Contract belongs to exactly one",
            several.len(),
            several
                .iter()
                .map(|engagement| format!("[[{}]]", engagement.name))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    Some(Finding::new(
        contract.file,
        Rule::Contract,
        At::Note,
        message,
    ))
}

/// `own-business` has no Contracts, so no Contract has a type to match.
const fn contract_type_of(engagement_type: EngagementType) -> Option<ContractType> {
    match engagement_type {
        EngagementType::Freelance => Some(ContractType::QuasiMandate),
        EngagementType::Employment => Some(ContractType::Employment),
        EngagementType::Dispatch => Some(ContractType::Dispatch),
        EngagementType::Internship => Some(ContractType::Internship),
        EngagementType::OwnBusiness => None,
    }
}

fn type_fixed_by(
    contract: &Found<'_, Contract>,
    engagement: &Found<'_, kariera_domain::Engagement>,
) -> Option<Finding> {
    let engagement_type = engagement.note.engagement_type;
    let expected = contract_type_of(engagement_type)?;
    (contract.note.contract_type != expected).then(|| {
        Finding::new(
            contract.file,
            Rule::Contract,
            At::Field(FieldPath::of("contract_type")),
            format!(
                "must be {} because the Engagement [[{}]] is {}",
                expected.slug(),
                engagement.name,
                engagement_type.slug()
            ),
        )
    })
}

#[cfg(test)]
mod tests {
    use crate::test_vault::TestVault;

    #[test]
    fn a_contract_that_no_engagement_lists_is_reported() {
        let vault = TestVault::new("contracts_unlisted");
        vault.replace(
            "Engagements/2023-10 Globex.md",
            "  - \"[[2025-04 Globex Contract]]\"\n",
            "",
        );

        assert_eq!(
            vault.lines(),
            [
                "Contracts/2025-04 Globex Contract.md: error [contract] the note is listed by no Engagement, but every Contract belongs to the Engagement that lists it"
            ]
        );
    }

    #[test]
    fn a_contract_that_two_engagements_list_is_reported() {
        let vault = TestVault::new("contracts_listed_twice");
        vault.replace(
            "Engagements/2018-07 Initech.md",
            "start: 2018-07\n",
            "start: 2018-07\ncontracts:\n  - \"[[2025-04 Globex Contract]]\"\n",
        );

        let lines = vault.lines();

        assert!(
            lines.contains(&"Contracts/2025-04 Globex Contract.md: error [contract] the note is listed by 2 Engagements ([[2018-07 Initech]], [[2023-10 Globex]]), but a Contract belongs to exactly one".to_owned()),
            "{lines:#?}"
        );
    }

    #[test]
    fn the_contract_type_follows_the_engagement_type() {
        let vault = TestVault::new("contracts_type");
        vault.replace(
            "Contracts/2019-04 Müller Logistik Contract.md",
            "contract_type: employment",
            "contract_type: internship",
        );

        assert_eq!(
            vault.lines(),
            [
                "Contracts/2019-04 Müller Logistik Contract.md:4: error [contract] field `contract_type` must be employment because the Engagement [[2019-04 Müller Logistik]] is employment"
            ]
        );
    }

    #[test]
    fn a_freelance_engagement_has_quasi_mandate_contracts() {
        let vault = TestVault::new("contracts_freelance");
        vault.replace(
            "Contracts/2025-04 Globex Contract.md",
            "contract_type: quasi-mandate\nstart: 2025-04-01\nrate: 640000\nrate_unit: month\n",
            "contract_type: dispatch\nstart: 2025-04-01\n",
        );

        assert_eq!(
            vault.lines(),
            [
                "Contracts/2025-04 Globex Contract.md:4: error [contract] field `contract_type` must be quasi-mandate because the Engagement [[2023-10 Globex]] is freelance"
            ]
        );
    }
}
