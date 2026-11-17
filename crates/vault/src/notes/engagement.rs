use kariera_domain::{Engagement, EngagementType, FixedList};

use crate::fields::{Fields, Presence};
use crate::problems::Reported;
use crate::sections::Sections;

/// Schema §2.2.
pub(super) fn read(
    fields: &mut Fields<'_>,
    sections: &mut Sections<'_>,
) -> Result<Engagement, Reported> {
    let engagement_type = fields.required::<EngagementType>("engagement_type");
    let client = fields.conditional("client", engagement_type.map(client_presence));
    let employer = fields.conditional("employer", engagement_type.map(employer_presence));
    let role = fields.optional("role");
    let start = fields.required("start");
    let end = fields.optional("end");
    let contracts = fields.conditional("contracts", engagement_type.map(contracts_presence));
    let translated_from = fields.translations("translated_from");
    let sources = fields.required("sources");
    let reviewed = fields.required("reviewed");
    let summary = sections.paragraphs("Summary");
    Ok(Engagement {
        client: client?,
        employer: employer?,
        engagement_type: engagement_type?,
        role: role?,
        start: start?,
        end: end?,
        contracts: contracts?.unwrap_or_default(),
        translated_from: translated_from?,
        sources: sources?,
        reviewed: reviewed?,
        summary: summary?,
    })
}

const fn works_for_a_client(engagement_type: EngagementType) -> bool {
    match engagement_type {
        EngagementType::Freelance | EngagementType::OwnBusiness => true,
        EngagementType::Employment | EngagementType::Dispatch | EngagementType::Internship => false,
    }
}

fn client_presence(engagement_type: EngagementType) -> Presence {
    if works_for_a_client(engagement_type) {
        Presence::Required(because(engagement_type))
    } else {
        Presence::Empty(because(engagement_type))
    }
}

fn employer_presence(engagement_type: EngagementType) -> Presence {
    if works_for_a_client(engagement_type) {
        Presence::Empty(because(engagement_type))
    } else {
        Presence::Required(because(engagement_type))
    }
}

fn contracts_presence(engagement_type: EngagementType) -> Presence {
    match engagement_type {
        EngagementType::Freelance => Presence::Required(because(engagement_type)),
        EngagementType::OwnBusiness => Presence::Empty(because(engagement_type)),
        EngagementType::Employment | EngagementType::Dispatch | EngagementType::Internship => {
            Presence::Optional
        }
    }
}

fn because(engagement_type: EngagementType) -> String {
    format!("engagement_type is {}", engagement_type.slug())
}
