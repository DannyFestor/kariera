//! How each schema field type (schema §1.2) is read from a YAML node. Scalars are read from their written text
//! whatever their YAML style, so `score: 980` is the Text `980`; whether a value had to be quoted is a question
//! of style, not of type.

use std::collections::BTreeMap;

use kariera_domain::{
    ContractType, Date, EngagementType, FixedList, Hours, Link, Month, Note, NoteName, NoteType,
    OrganizationKind, Phase, QualificationCategory, RateStep, RateUnit, SkillCategory, Source,
    SourceKind, SourcePath, Text, Translated, Translations, Url, ValueError, WholeNumber,
};

use crate::Problem;
use crate::cursor::Cursor;
use crate::fields::{Presence, read_map};
use crate::names::nfc;
use crate::problems::Problems;
use crate::yaml::{Node, Value};

pub(crate) trait FromNode: Sized {
    /// Only called for a node that is not empty.
    fn from_node(node: &Node, cursor: &Cursor<'_>) -> Result<Self, Vec<Problem>>;

    fn is_empty(node: &Node) -> bool {
        node.is_empty()
    }
}

/// A type written as one YAML scalar. The error completes "field `x` …".
pub(crate) trait FromScalar: Sized {
    fn from_scalar(text: &str) -> Result<Self, String>;
}

impl<T: FromScalar> FromNode for T {
    fn from_node(node: &Node, cursor: &Cursor<'_>) -> Result<Self, Vec<Problem>> {
        scalar_text(node)
            .and_then(T::from_scalar)
            .map_err(|message| vec![cursor.problem(node.line, message)])
    }
}

pub(crate) fn scalar_text(node: &Node) -> Result<&str, String> {
    match &node.value {
        Value::Scalar { text, .. } => Ok(text),
        Value::Sequence(_) | Value::Mapping(_) => {
            Err("must be a single value, not a list or map".to_owned())
        }
    }
}

macro_rules! parsed_scalars {
    ($($type:ty),+ $(,)?) => {
        $(impl FromScalar for $type {
            fn from_scalar(text: &str) -> Result<Self, String> {
                text.parse().map_err(|error: ValueError| error.to_string())
            }
        })+
    };
}

parsed_scalars!(Text, Url, Month, Date, Hours, SourcePath);

impl<const MIN: u32, const MAX: u32> FromScalar for WholeNumber<MIN, MAX> {
    fn from_scalar(text: &str) -> Result<Self, String> {
        text.parse().map_err(|error: ValueError| error.to_string())
    }
}

macro_rules! slug_scalars {
    ($($type:ty),+ $(,)?) => {
        $(impl FromScalar for $type {
            fn from_scalar(text: &str) -> Result<Self, String> {
                slug(text)
            }
        })+
    };
}

slug_scalars!(
    NoteType,
    OrganizationKind,
    EngagementType,
    ContractType,
    RateUnit,
    Phase,
    SkillCategory,
    QualificationCategory,
    SourceKind,
);

fn slug<T: FixedList>(text: &str) -> Result<T, String> {
    T::from_slug(text).ok_or_else(|| format!("must be one of {}", T::slugs().join(", ")))
}

impl FromScalar for bool {
    fn from_scalar(text: &str) -> Result<Self, String> {
        match text {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err("must be true or false".to_owned()),
        }
    }
}

impl<T: Note> FromNode for Link<T> {
    fn from_node(node: &Node, cursor: &Cursor<'_>) -> Result<Self, Vec<Problem>> {
        scalar_text(node)
            .and_then(|text| link(text, cursor))
            .map_err(|message| vec![cursor.problem(node.line, message)])
    }
}

fn link<T: Note>(text: &str, cursor: &Cursor<'_>) -> Result<Link<T>, String> {
    let target = text
        .strip_prefix("[[")
        .and_then(|rest| rest.strip_suffix("]]"))
        .ok_or_else(|| format!("must be a link such as \"[[Note name]]\", not {text:?}"))?;
    let name = nfc(target)
        .parse::<NoteName>()
        .map_err(|error| format!("must link a note by its name, which {error}"))?;
    cursor.names().link(name)
}

/// A List (schema §1.2): no empty entries, no duplicates.
impl<T: FromNode + PartialEq> FromNode for Vec<T> {
    fn from_node(node: &Node, cursor: &Cursor<'_>) -> Result<Self, Vec<Problem>> {
        let Value::Sequence(items) = &node.value else {
            return Err(vec![cursor.problem(node.line, "must be a list")]);
        };
        let problems = Problems::default();
        let mut values = Self::new();
        for (index, item) in items.iter().enumerate() {
            let cursor = cursor.item(index);
            if T::is_empty(item) {
                problems.record(cursor.problem(item.line, "must not be empty"));
            } else if let Ok(value) = problems.adopt(T::from_node(item, &cursor)) {
                if values.contains(&value) {
                    problems.record(cursor.problem(item.line, "repeats an earlier entry"));
                } else {
                    values.push(value);
                }
            }
        }
        problems.into_result(Ok(values))
    }
}

/// A map from a link to a value, such as `project_hours` (schema §1.11).
impl<T: Note, V: FromNode> FromNode for BTreeMap<Link<T>, V> {
    fn from_node(node: &Node, cursor: &Cursor<'_>) -> Result<Self, Vec<Problem>> {
        let Value::Mapping(entries) = &node.value else {
            return Err(vec![cursor.problem(node.line, "must be a map")]);
        };
        let problems = Problems::default();
        let mut map = Self::new();
        for entry in entries {
            let cursor = cursor.key(&entry.key);
            let line = entry.value.line;
            let target =
                link(&entry.key, &cursor).map_err(|message| vec![cursor.problem(line, message)]);
            let value = if V::is_empty(&entry.value) {
                Err(vec![cursor.problem(line, "is required")])
            } else {
                V::from_node(&entry.value, &cursor)
            };
            if let (Ok(target), Ok(value)) = (problems.adopt(target), problems.adopt(value))
                && map.insert(target, value).is_some()
            {
                problems.record(cursor.problem(line, "links the same note as an earlier key"));
            }
        }
        problems.into_result(Ok(map))
    }
}

/// A Map3 (schema §1.5): `en` is required as soon as any language is set.
impl FromNode for Translated<Text> {
    fn from_node(node: &Node, cursor: &Cursor<'_>) -> Result<Self, Vec<Problem>> {
        read_map(node, cursor, |fields| {
            let en = fields.required("en");
            let ja = fields.optional("ja");
            let de = fields.optional("de");
            Ok(Self {
                en: en?,
                ja: ja?,
                de: de?,
            })
        })
    }

    fn is_empty(node: &Node) -> bool {
        every_value_is_empty(node)
    }
}

impl FromNode for Translations<Text> {
    fn from_node(node: &Node, cursor: &Cursor<'_>) -> Result<Self, Vec<Problem>> {
        read_map(node, cursor, |fields| {
            let ja = fields.optional("ja");
            let de = fields.optional("de");
            Ok(Self { ja: ja?, de: de? })
        })
    }

    fn is_empty(node: &Node) -> bool {
        every_value_is_empty(node)
    }
}

fn every_value_is_empty(node: &Node) -> bool {
    match &node.value {
        Value::Mapping(entries) => entries.iter().all(|entry| entry.value.is_empty()),
        Value::Scalar { .. } | Value::Sequence(_) => node.is_empty(),
    }
}

/// Schema §1.7.
impl FromNode for Source {
    fn from_node(map: &Node, cursor: &Cursor<'_>) -> Result<Self, Vec<Problem>> {
        read_map(map, cursor, |fields| {
            let kind = fields.required::<SourceKind>("kind");
            let path = fields.conditional("path", kind.map(path_presence));
            let note = fields.conditional("note", kind.map(note_presence));
            Ok(Self {
                kind: kind?,
                path: path?,
                note: note?,
            })
        })
    }
}

fn path_presence(kind: SourceKind) -> Presence {
    match kind {
        SourceKind::Recalled => Presence::Empty("kind is recalled".to_owned()),
        SourceKind::WorkReport
        | SourceKind::Invoice
        | SourceKind::Contract
        | SourceKind::PurchaseOrder
        | SourceKind::Timesheet
        | SourceKind::SkillSheet
        | SourceKind::Cv
        | SourceKind::TrackerDigest
        | SourceKind::Certificate => Presence::Required(format!("kind is {}", kind.slug())),
    }
}

fn note_presence(kind: SourceKind) -> Presence {
    if kind == SourceKind::Recalled {
        Presence::Required("kind is recalled".to_owned())
    } else {
        Presence::Optional
    }
}

/// Schema §1.10.
impl FromNode for RateStep {
    fn from_node(node: &Node, cursor: &Cursor<'_>) -> Result<Self, Vec<Problem>> {
        read_map(node, cursor, |fields| {
            let from = fields.required("from");
            let rate = fields.required("rate");
            Ok(Self {
                from: from?,
                rate: rate?,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use kariera_domain::{
        DaysPerWeek, Hours, Link, NoteType, Phase, Project, RateStep, Skill, Source, SourceKind,
        Text, Translated, Translations,
    };

    use super::FromNode;
    use crate::cursor::Cursor;
    use crate::names::Names;
    use crate::yaml::{self, Node};
    use crate::{Location, Problem};

    fn names() -> Names {
        let mut names = Names::default();
        names.insert("Müller".parse().unwrap(), Some(NoteType::Skill));
        names.insert("Acme - Shop".parse().unwrap(), Some(NoteType::Project));
        names.insert("Acme - Admin".parse().unwrap(), Some(NoteType::Project));
        names
    }

    fn value_node(yaml: &str) -> Node {
        let root = yaml::parse(yaml, 1).unwrap().unwrap();
        root.get("value").unwrap().clone()
    }

    /// Reads the value of the key `value`, as a field of that name.
    fn read<T: FromNode>(yaml: &str) -> Result<T, Vec<Problem>> {
        T::from_node(&value_node(yaml), &Cursor::root(&names()).key("value"))
    }

    fn problem(field: &str, line: usize, message: &str) -> Problem {
        Problem::new(Location::Field(field.to_owned()), Some(line), message)
    }

    fn text(value: &str) -> Text {
        value.parse().unwrap()
    }

    fn link<T>(name: &str) -> Link<T> {
        Link::new(name.parse().unwrap())
    }

    #[test]
    fn text_is_read_from_the_written_scalar_whatever_its_yaml_type() {
        assert_eq!(read::<Text>("value: 980\n"), Ok(text("980")));
        assert_eq!(read::<Text>("value: \"true\"\n"), Ok(text("true")));
    }

    #[test]
    fn a_scalar_field_refuses_a_list() {
        assert_eq!(
            read::<Text>("value:\n  - a\n"),
            Err(vec![problem(
                "value",
                2,
                "must be a single value, not a list or map"
            )])
        );
    }

    #[test]
    fn a_value_error_is_reported_with_its_field() {
        assert_eq!(
            read::<Hours>("value: 1.234\n"),
            Err(vec![problem(
                "value",
                1,
                "must be a number of hours with at most 2 decimals, such as 152.5"
            )])
        );
        assert_eq!(
            read::<DaysPerWeek>("value: 9\n"),
            Err(vec![problem(
                "value",
                1,
                "must be a whole number from 1 to 7"
            )])
        );
    }

    #[test]
    fn an_enum_is_read_from_its_slug() {
        assert_eq!(
            read::<Phase>("value: basic-design\n"),
            Ok(Phase::BasicDesign)
        );
        assert_eq!(
            read::<SourceKind>("value: email\n"),
            Err(vec![problem(
                "value",
                1,
                "must be one of work-report, invoice, contract, purchase-order, timesheet, \
                 skill-sheet, cv, tracker-digest, certificate, recalled"
            )])
        );
    }

    #[test]
    fn a_bool_is_true_or_false() {
        assert_eq!(read::<bool>("value: true\n"), Ok(true));
        assert_eq!(read::<bool>("value: false\n"), Ok(false));
        assert_eq!(
            read::<bool>("value: yes\n"),
            Err(vec![problem("value", 1, "must be true or false")])
        );
    }

    #[test]
    fn a_link_resolves_to_a_note_of_its_type() {
        assert_eq!(
            read::<Link<Skill>>("value: \"[[Müller]]\"\n"),
            Ok(link("Müller"))
        );
    }

    #[test]
    fn a_link_target_in_nfd_resolves_to_the_nfc_name() {
        assert_eq!(
            read::<Link<Skill>>("value: \"[[Mu\u{308}ller]]\"\n"),
            Ok(link("Müller"))
        );
    }

    #[test]
    fn a_link_must_be_a_wikilink_to_a_plain_name() {
        assert_eq!(
            read::<Link<Skill>>("value: Müller\n"),
            Err(vec![problem(
                "value",
                1,
                "must be a link such as \"[[Note name]]\", not \"Müller\""
            )])
        );
        assert_eq!(
            read::<Link<Skill>>("value: \"[[Müller|M]]\"\n"),
            Err(vec![problem(
                "value",
                1,
                "must link a note by its name, which must not contain '|'"
            )])
        );
    }

    #[test]
    fn a_link_to_the_wrong_type_is_reported() {
        assert_eq!(
            read::<Link<Project>>("value: \"[[Müller]]\"\n"),
            Err(vec![problem(
                "value",
                1,
                "links to [[Müller]], whose type is skill, not project"
            )])
        );
    }

    #[test]
    fn a_list_reads_every_entry() {
        let phases = read::<Vec<Phase>>("value:\n  - implementation\n  - testing\n");

        assert_eq!(phases, Ok(vec![Phase::Implementation, Phase::Testing]));
    }

    #[test]
    fn a_list_refuses_empty_repeated_and_bad_entries() {
        let phases = read::<Vec<Phase>>("value:\n  - testing\n  -\n  - testing\n  - coding\n");

        assert_eq!(
            phases,
            Err(vec![
                problem("value[1]", 3, "must not be empty"),
                problem("value[2]", 4, "repeats an earlier entry"),
                problem(
                    "value[3]",
                    5,
                    "must be one of requirements, basic-design, detailed-design, implementation, \
                     testing, operations"
                ),
            ])
        );
    }

    #[test]
    fn a_list_field_refuses_a_single_value() {
        assert_eq!(
            read::<Vec<Phase>>("value: testing\n"),
            Err(vec![problem("value", 1, "must be a list")])
        );
    }

    #[test]
    fn project_hours_map_links_to_hours() {
        let hours = read::<BTreeMap<Link<Project>, Hours>>(
            "value:\n  \"[[Acme - Shop]]\": 120\n  \"[[Acme - Admin]]\": 30.5\n",
        );

        assert_eq!(
            hours,
            Ok(BTreeMap::from([
                (link("Acme - Admin"), "30.5".parse().unwrap()),
                (link("Acme - Shop"), "120".parse().unwrap()),
            ]))
        );
    }

    #[test]
    fn project_hours_refuse_bad_keys_values_and_repeats() {
        let hours = read::<BTreeMap<Link<Project>, Hours>>(
            "value:\n  \"[[Acme - Shop]]\": 1\n  \"[[Nobody]]\": 2\n  \"[[Acme - Admin]]\":\n  \
             \"[[Acme - Shop ]]\": 3\n",
        );

        assert_eq!(
            hours,
            Err(vec![
                problem(
                    "value.[[Nobody]]",
                    3,
                    "links to [[Nobody]], but no note has that name"
                ),
                problem("value.[[Acme - Admin]]", 4, "is required"),
                problem(
                    "value.[[Acme - Shop ]]",
                    5,
                    "must link a note by its name, which must not start or end with a space or a dot"
                ),
            ])
        );
        assert_eq!(
            read::<BTreeMap<Link<Project>, Hours>>("value: 5\n"),
            Err(vec![problem("value", 1, "must be a map")])
        );
    }

    #[test]
    fn project_hours_refuse_two_keys_for_one_note() {
        let normalized = read::<BTreeMap<Link<Skill>, Hours>>(
            "value:\n  \"[[Müller]]\": 1\n  \"[[Mu\u{308}ller]]\": 2\n",
        );
        assert_eq!(
            normalized,
            Err(vec![problem(
                "value.[[Mu\u{308}ller]]",
                3,
                "links the same note as an earlier key"
            )])
        );
    }

    #[test]
    fn a_map3_reads_each_language() {
        let role = read::<Translated<Text>>("value:\n  en: Developer\n  ja: 開発者\n  de:\n");

        assert_eq!(
            role,
            Ok(Translated {
                en: text("Developer"),
                ja: Some(text("開発者")),
                de: None,
            })
        );
    }

    #[test]
    fn a_map3_needs_english_once_any_language_is_set() {
        assert_eq!(
            read::<Translated<Text>>("value:\n  en:\n  ja: 開発者\n"),
            Err(vec![problem("value.en", 2, "is required")])
        );
    }

    #[test]
    fn a_map3_refuses_other_languages() {
        assert_eq!(
            read::<Translated<Text>>("value:\n  en: Developer\n  fr: Développeur\n"),
            Err(vec![problem(
                "value.fr",
                3,
                "is not a field the schema allows here"
            )])
        );
    }

    #[test]
    fn a_translatable_map_with_only_empty_keys_is_empty() {
        let root = yaml::parse("a:\n  en:\n  ja:\n  de:\nb: {}\nc: x\n", 1)
            .unwrap()
            .unwrap();

        let empty = ["a", "b", "c"].map(|key| {
            let node = root.get(key).unwrap();
            (
                Translated::<Text>::is_empty(node),
                Translations::<Text>::is_empty(node),
            )
        });

        assert_eq!(empty, [(true, true), (true, true), (false, false)]);
    }

    #[test]
    fn translations_read_japanese_and_german() {
        assert_eq!(
            read::<Translations<Text>>("value:\n  ja: 3f9a1c0b7e2d\n  de: 123456\n"),
            Ok(Translations {
                ja: Some(text("3f9a1c0b7e2d")),
                de: Some(text("123456")),
            })
        );
    }

    #[test]
    fn a_source_with_a_file_has_a_path() {
        let source = read::<Source>("value:\n  kind: invoice\n  path: in/a.pdf\n  note: 150 h\n");

        assert_eq!(
            source,
            Ok(Source {
                kind: SourceKind::Invoice,
                path: Some("in/a.pdf".parse().unwrap()),
                note: Some(text("150 h")),
            })
        );
    }

    #[test]
    fn a_source_with_a_file_needs_its_path() {
        assert_eq!(
            read::<Source>("value:\n  kind: timesheet\n"),
            Err(vec![problem(
                "value.path",
                2,
                "is required because kind is timesheet"
            )])
        );
    }

    #[test]
    fn a_recalled_source_has_a_note_and_no_path() {
        assert_eq!(
            read::<Source>("value:\n  kind: recalled\n  note: \"owner: about 80%\"\n"),
            Ok(Source {
                kind: SourceKind::Recalled,
                path: None,
                note: Some(text("owner: about 80%")),
            })
        );
        assert_eq!(
            read::<Source>("value:\n  kind: recalled\n  path: in/a.pdf\n"),
            Err(vec![
                problem("value.path", 3, "must be empty because kind is recalled"),
                problem("value.note", 2, "is required because kind is recalled"),
            ])
        );
    }

    #[test]
    fn a_rate_step_has_a_month_and_a_rate() {
        assert_eq!(
            read::<RateStep>("value:\n  from: 2025-06\n  rate: 4500\n"),
            Ok(RateStep {
                from: "2025-06".parse().unwrap(),
                rate: "4500".parse().unwrap(),
            })
        );
    }
}
