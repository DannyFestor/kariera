//! The frontmatter of each note type as the schema's tables write it (§2): the fields in order, and how each
//! value is written (§1.2).

use kariera_domain::NoteType;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Field {
    pub(crate) key: &'static str,
    pub(crate) shape: Shape,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Shape {
    /// A string, written plain unless YAML requires quotes.
    Text,
    /// A month, date, number, boolean or enum slug.
    Unquoted,
    /// A wikilink, always double-quoted.
    Link,
    /// A value whose format another part of kariera owns: the fingerprints of `translated_from` (§1.6.1).
    Free,
    List(&'static Self),
    /// A map with these keys, in this order.
    Map(&'static [Field]),
    /// A map from links, which are double-quoted keys, to values (`project_hours`).
    LinkKeyed(&'static Self),
}

const fn field(key: &'static str, shape: Shape) -> Field {
    Field { key, shape }
}

const MAP3: Shape = Shape::Map(&[
    field("en", Shape::Text),
    field("ja", Shape::Text),
    field("de", Shape::Text),
]);
const LABELS: Shape = Shape::Map(&[field("ja", Shape::Text), field("de", Shape::Text)]);
const TRANSLATED_FROM: Shape = Shape::Map(&[field("ja", Shape::Free), field("de", Shape::Free)]);
const SOURCES: Shape = Shape::List(&Shape::Map(&[
    field("kind", Shape::Unquoted),
    field("path", Shape::Text),
    field("note", Shape::Text),
]));
const RATE_HISTORY: Shape = Shape::List(&Shape::Map(&[
    field("from", Shape::Unquoted),
    field("rate", Shape::Unquoted),
]));
const TEXTS: Shape = Shape::List(&Shape::Text);
const UNQUOTED_LIST: Shape = Shape::List(&Shape::Unquoted);
const LINKS: Shape = Shape::List(&Shape::Link);

const TYPE: Field = field("type", Shape::Unquoted);
const REVIEWED: Field = field("reviewed", Shape::Unquoted);

const ORGANIZATION: &[Field] = &[
    TYPE,
    field("name", Shape::Text),
    field("aliases", TEXTS),
    field("kind", UNQUOTED_LIST),
    field("location", Shape::Text),
    field("website", Shape::Text),
    field("labels", LABELS),
    field("translated_from", TRANSLATED_FROM),
    field("sources", SOURCES),
    REVIEWED,
];

const ENGAGEMENT: &[Field] = &[
    TYPE,
    field("client", Shape::Link),
    field("employer", Shape::Link),
    field("engagement_type", Shape::Unquoted),
    field("role", MAP3),
    field("start", Shape::Unquoted),
    field("end", Shape::Unquoted),
    field("contracts", LINKS),
    field("translated_from", TRANSLATED_FROM),
    field("sources", SOURCES),
    REVIEWED,
];

const CONTRACT: &[Field] = &[
    TYPE,
    field("counterparty", Shape::Link),
    field("contract_type", Shape::Unquoted),
    field("start", Shape::Unquoted),
    field("end", Shape::Unquoted),
    field("rate", Shape::Unquoted),
    field("rate_unit", Shape::Unquoted),
    field("rate_history", RATE_HISTORY),
    field("sources", SOURCES),
];

const PROJECT: &[Field] = &[
    TYPE,
    field("engagement", Shape::Link),
    field("end_client", Shape::Link),
    field("title", MAP3),
    field("public_title", MAP3),
    field("confidential", Shape::Unquoted),
    field("aliases", TEXTS),
    field("start", Shape::Unquoted),
    field("end", Shape::Unquoted),
    field("role", MAP3),
    field("focus", MAP3),
    field("team_size", Shape::Unquoted),
    field("industry", MAP3),
    field("phases", UNQUOTED_LIST),
    field("skills", LINKS),
    field("translated_from", TRANSLATED_FROM),
    field("sources", SOURCES),
    REVIEWED,
];

const MONTHLY_LOG: &[Field] = &[
    TYPE,
    field("engagement", Shape::Link),
    field("month", Shape::Unquoted),
    field("hours", Shape::Unquoted),
    field("project_hours", Shape::LinkKeyed(&Shape::Unquoted)),
    field("sources", SOURCES),
];

const SKILL: &[Field] = &[
    TYPE,
    field("name", Shape::Text),
    field("aliases", TEXTS),
    field("category", Shape::Unquoted),
    field("labels", LABELS),
    field("level", MAP3),
    field("last_used", Shape::Unquoted),
    field("translated_from", TRANSLATED_FROM),
    field("sources", SOURCES),
    REVIEWED,
];

const QUALIFICATION: &[Field] = &[
    TYPE,
    field("name", Shape::Text),
    field("labels", LABELS),
    field("category", Shape::Unquoted),
    field("issuer", Shape::Text),
    field("start", Shape::Unquoted),
    field("date", Shape::Unquoted),
    field("level", Shape::Text),
    field("score", Shape::Text),
    field("translated_from", TRANSLATED_FROM),
    field("sources", SOURCES),
    REVIEWED,
];

const PROFILE: &[Field] = &[
    TYPE,
    field("name", Shape::Text),
    field("headline", MAP3),
    field("business", Shape::Link),
    field("available_from", Shape::Unquoted),
    field("days_per_week_min", Shape::Unquoted),
    field("days_per_week_max", Shape::Unquoted),
    field("portfolio_url", Shape::Text),
    field("github_url", Shape::Text),
    field("translated_from", TRANSLATED_FROM),
    field("sources", SOURCES),
    REVIEWED,
];

/// The fields of a note type, in the order of its schema table.
pub(crate) const fn fields_of(note_type: NoteType) -> &'static [Field] {
    match note_type {
        NoteType::Organization => ORGANIZATION,
        NoteType::Engagement => ENGAGEMENT,
        NoteType::Contract => CONTRACT,
        NoteType::Project => PROJECT,
        NoteType::MonthlyLog => MONTHLY_LOG,
        NoteType::Skill => SKILL,
        NoteType::Qualification => QUALIFICATION,
        NoteType::Profile => PROFILE,
    }
}
