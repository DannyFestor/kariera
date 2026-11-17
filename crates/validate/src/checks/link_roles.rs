//! Schema §3 (link roles): a link to an Organization needs an Organization of the right kind.

use kariera_domain::{
    Contract, Engagement, EngagementType, FixedList, Link, Organization, OrganizationKind, Profile,
    Project,
};

use crate::context::{Context, Found, client_link};
use crate::field_path::FieldPath;
use crate::finding::{At, Finding, Rule};

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(
        context
            .notes::<Engagement>()
            .flat_map(|e| engagement(context, &e)),
    );
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
            .notes::<Profile>()
            .filter_map(|p| profile(context, &p)),
    );
    findings
}

/// The kinds an Organization may have where it is linked from `field`.
struct Role {
    field: &'static str,
    name: &'static str,
    accepted: &'static [OrganizationKind],
}

fn engagement(context: &Context<'_>, engagement: &Found<'_, Engagement>) -> Vec<Finding> {
    let client_kinds: &[OrganizationKind] = match engagement.note.engagement_type {
        EngagementType::OwnBusiness => &[OrganizationKind::MyBusiness],
        EngagementType::Freelance
        | EngagementType::Employment
        | EngagementType::Dispatch
        | EngagementType::Internship => &[OrganizationKind::Client],
    };
    let client = Role {
        field: "client",
        name: "Client",
        accepted: client_kinds,
    };
    let employer = Role {
        field: "employer",
        name: "employer",
        accepted: &[OrganizationKind::Employer],
    };
    [
        (&engagement.note.client, client),
        (&engagement.note.employer, employer),
    ]
    .into_iter()
    .filter_map(|(link, role)| {
        link.as_ref()
            .and_then(|link| link_role(context, engagement.file, link, &role))
    })
    .collect()
}

fn project(context: &Context<'_>, project: &Found<'_, Project>) -> Vec<Finding> {
    let Some(end_client) = &project.note.end_client else {
        return Vec::new();
    };
    let role = Role {
        field: "end_client",
        name: "End Client",
        accepted: &[OrganizationKind::EndClient],
    };
    end_client_is_not_the_client(context, project, end_client)
        .or_else(|| link_role(context, project.file, end_client, &role))
        .into_iter()
        .collect()
}

/// `end_client` is set only if it differs from the Client (schema §2.4).
fn end_client_is_not_the_client(
    context: &Context<'_>,
    project: &Found<'_, Project>,
    end_client: &Link<Organization>,
) -> Option<Finding> {
    let engagement = context.get(&project.note.engagement)?;
    (client_link(engagement.note)? == end_client).then(|| {
        Finding::new(
            project.file,
            Rule::LinkRole,
            At::Field(FieldPath::of("end_client")),
            format!(
                "must be empty: [[{}]] is the Client of the Engagement, and an End Client is set only if it differs",
                end_client.target()
            ),
        )
    })
}

fn contract(context: &Context<'_>, contract: &Found<'_, Contract>) -> Vec<Finding> {
    let counterparty = &contract.note.counterparty;
    let role = Role {
        field: "counterparty",
        name: "counterparty",
        accepted: &[
            OrganizationKind::Client,
            OrganizationKind::Employer,
            OrganizationKind::Agency,
        ],
    };
    link_role(context, contract.file, counterparty, &role)
        .or_else(|| agency_unless_the_client(context, contract))
        .into_iter()
        .collect()
}

/// A counterparty that is not the Engagement's Client or employer is an Agency (schema §2.3).
fn agency_unless_the_client(
    context: &Context<'_>,
    contract: &Found<'_, Contract>,
) -> Option<Finding> {
    let engagement = context.engagement_of(contract)?;
    let client = client_link(engagement.note)?;
    let counterparty = &contract.note.counterparty;
    let organization = context.get(counterparty)?;
    let is_agency = organization.note.kind.contains(&OrganizationKind::Agency);
    (counterparty != client && !is_agency).then(|| {
        Finding::new(
            contract.file,
            Rule::LinkRole,
            At::Field(FieldPath::of("counterparty")),
            format!(
                "links [[{}]], which is not the Client or employer of [[{}]] and not an Agency (its kind is {})",
                counterparty.target(),
                engagement.name,
                slugs(&organization.note.kind, " and ")
            ),
        )
    })
}

fn profile(context: &Context<'_>, profile: &Found<'_, Profile>) -> Option<Finding> {
    let role = Role {
        field: "business",
        name: "business",
        accepted: &[OrganizationKind::MyBusiness],
    };
    link_role(context, profile.file, &profile.note.business, &role)
}

fn link_role(
    context: &Context<'_>,
    file: &kariera_vault::NoteFile,
    link: &Link<Organization>,
    role: &Role,
) -> Option<Finding> {
    let organization = context.get(link)?;
    let fits = role
        .accepted
        .iter()
        .any(|kind| organization.note.kind.contains(kind));
    (!fits).then(|| {
        Finding::new(
            file,
            Rule::LinkRole,
            At::Field(FieldPath::of(role.field)),
            format!(
                "links [[{}]], whose kind is {}, but the {} must have the kind {}",
                organization.name,
                slugs(&organization.note.kind, " and "),
                role.name,
                slugs(role.accepted, " or ")
            ),
        )
    })
}

fn slugs(kinds: &[OrganizationKind], conjunction: &str) -> String {
    let slugs: Vec<&str> = kinds.iter().map(|kind| kind.slug()).collect();
    match slugs.split_last() {
        Some((last, [])) => (*last).to_owned(),
        Some((last, first)) => format!("{}{conjunction}{last}", first.join(", ")),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use crate::test_vault::{TestVault, assert_reports};

    #[test]
    fn a_client_must_be_an_organization_of_kind_client() {
        let vault = TestVault::new("roles_client");
        vault.replace(
            "Organizations/Acme.md",
            "  - client\n  - agency",
            "  - agency",
        );

        assert_eq!(
            vault.lines(),
            [
                "Engagements/2021-04 Acme.md:3: error [link-role] field `client` links [[Acme]], whose kind is agency, but the Client must have the kind client"
            ]
        );
    }

    #[test]
    fn the_client_of_an_own_business_engagement_is_my_business() {
        let vault = TestVault::new("roles_own_business");
        vault.replace(
            "Engagements/2023-10 Globex.md",
            "engagement_type: freelance",
            "engagement_type: own-business",
        );
        vault.replace(
            "Engagements/2023-10 Globex.md",
            "contracts:\n  - \"[[2023-10 Globex via Fabrikam Contract]]\"\n  - \"[[2025-04 Globex Contract]]\"\n",
            "",
        );

        assert_reports(
            &vault,
            "Engagements/2023-10 Globex.md:3: error [link-role] field `client` links [[Globex]], whose kind is client, but the Client must have the kind my-business",
        );
    }

    #[test]
    fn an_employer_must_be_an_organization_of_kind_employer() {
        let vault = TestVault::new("roles_employer");
        vault.replace(
            "Organizations/Müller Logistik.md",
            "  - employer",
            "  - school",
        );

        assert_eq!(
            vault.lines(),
            [
                "Contracts/2019-04 Müller Logistik Contract.md:3: error [link-role] field `counterparty` links [[Müller Logistik]], whose kind is school, but the counterparty must have the kind client, employer or agency",
                "Engagements/2019-04 Müller Logistik.md:3: error [link-role] field `employer` links [[Müller Logistik]], whose kind is school, but the employer must have the kind employer",
            ]
        );
    }

    #[test]
    fn an_end_client_must_be_an_organization_of_kind_end_client() {
        let vault = TestVault::new("roles_end_client");
        vault.replace("Organizations/Northwind.md", "  - end-client", "  - client");

        assert_eq!(
            vault.lines(),
            [
                "Projects/Acme - Ledger.md:4: error [link-role] field `end_client` links [[Northwind]], whose kind is client, but the End Client must have the kind end-client"
            ]
        );
    }

    #[test]
    fn an_end_client_must_differ_from_the_client() {
        let vault = TestVault::new("roles_end_client_is_client");
        vault.replace(
            "Projects/Acme - Shop.md",
            "end_client: \"[[Contoso]]\"",
            "end_client: \"[[Acme]]\"",
        );

        assert_eq!(
            vault.lines(),
            [
                "Projects/Acme - Shop.md:4: error [link-role] field `end_client` must be empty: [[Acme]] is the Client of the Engagement, and an End Client is set only if it differs",
            ]
        );
    }

    #[test]
    fn a_counterparty_that_is_not_the_client_must_be_an_agency() {
        let vault = TestVault::new("roles_agency");
        vault.replace("Organizations/Fabrikam.md", "  - agency", "  - client");

        assert_eq!(
            vault.lines(),
            [
                "Contracts/2023-10 Globex via Fabrikam Contract.md:3: error [link-role] field `counterparty` links [[Fabrikam]], which is not the Client or employer of [[2023-10 Globex]] and not an Agency (its kind is client)"
            ]
        );
    }

    #[test]
    fn a_counterparty_must_be_a_client_an_employer_or_an_agency() {
        let vault = TestVault::new("roles_counterparty");
        vault.replace("Organizations/Fabrikam.md", "  - agency", "  - school");

        assert_eq!(
            vault.lines(),
            [
                "Contracts/2023-10 Globex via Fabrikam Contract.md:3: error [link-role] field `counterparty` links [[Fabrikam]], whose kind is school, but the counterparty must have the kind client, employer or agency"
            ]
        );
    }

    #[test]
    fn the_business_of_the_profile_is_my_business() {
        let vault = TestVault::new("roles_business");
        vault.replace(
            "Organizations/Example Works.md",
            "  - my-business",
            "  - client",
        );

        assert_eq!(
            vault.lines(),
            [
                "Profile.md:8: error [link-role] field `business` links [[Example Works]], whose kind is client, but the business must have the kind my-business"
            ]
        );
    }
}
