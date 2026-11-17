use kariera_domain::Translations;

use crate::Problem;
use crate::cursor::Cursor;
use crate::problems::{Problems, Reported};
use crate::value::FromNode;
use crate::yaml::{Entry, Node, Value};

/// Reads the fields of one YAML map, recording every problem, and at the end every key it never asked for.
#[derive(Debug)]
pub(crate) struct Fields<'a> {
    line: usize,
    entries: &'a [Entry],
    asked: Vec<&'a str>,
    cursor: Cursor<'a>,
    problems: &'a Problems,
}

/// Whether a conditional field (Req `C` in the schema) must be set, may be set or must be empty, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Presence {
    Required(String),
    Optional,
    Empty(String),
}

impl<'a> Fields<'a> {
    pub(crate) fn new(
        node: &'a Node,
        cursor: Cursor<'a>,
        problems: &'a Problems,
    ) -> Result<Self, Reported> {
        let entries = match &node.value {
            Value::Mapping(entries) => entries.as_slice(),
            Value::Scalar { .. } | Value::Sequence(_) if node.is_empty() => &[],
            Value::Scalar { .. } | Value::Sequence(_) => {
                problems.record(cursor.problem(node.line, "must be a map of fields"));
                return Err(Reported);
            }
        };
        Ok(Self {
            line: node.line,
            entries,
            asked: Vec::new(),
            cursor,
            problems,
        })
    }

    pub(crate) fn required<T: FromNode>(&mut self, key: &'a str) -> Result<T, Reported> {
        self.ask::<T>(key).map_or_else(
            || Err(self.missing(key, "is required")),
            |node| self.read(key, node),
        )
    }

    pub(crate) fn optional<T: FromNode>(&mut self, key: &'a str) -> Result<Option<T>, Reported> {
        self.ask::<T>(key)
            .map(|node| self.read(key, node))
            .transpose()
    }

    pub(crate) fn list<T>(&mut self, key: &'a str) -> Result<Vec<T>, Reported>
    where
        Vec<T>: FromNode,
    {
        self.optional(key).map(Option::unwrap_or_default)
    }

    pub(crate) fn translations<T>(&mut self, key: &'a str) -> Result<Translations<T>, Reported>
    where
        Translations<T>: FromNode,
    {
        self.optional(key)
            .map(|translations| translations.unwrap_or_else(Translations::none))
    }

    /// `presence` is an error when the field it depends on could not be read; this field is then not checked.
    pub(crate) fn conditional<T: FromNode>(
        &mut self,
        key: &'a str,
        presence: Result<Presence, Reported>,
    ) -> Result<Option<T>, Reported> {
        let node = self.ask::<T>(key);
        match (presence?, node) {
            (Presence::Required(why), None) => {
                Err(self.missing(key, &format!("is required because {why}")))
            }
            (Presence::Empty(why), Some(node)) => {
                let problem = self
                    .cursor
                    .key(key)
                    .problem(node.line, format!("must be empty because {why}"));
                self.problems.record(problem);
                Err(Reported)
            }
            (Presence::Required(_) | Presence::Optional, Some(node)) => {
                self.read(key, node).map(Some)
            }
            (Presence::Optional | Presence::Empty(_), None) => Ok(None),
        }
    }

    /// A key read before the fields of its note, which is not unknown.
    pub(crate) fn skip(&mut self, key: &'a str) {
        self.asked.push(key);
    }

    pub(crate) fn finish(self) {
        for entry in self.entries {
            if !self.asked.contains(&entry.key.as_str()) {
                let problem = self
                    .cursor
                    .key(&entry.key)
                    .problem(entry.value.line, "is not a field the schema allows here");
                self.problems.record(problem);
            }
        }
    }

    /// An empty value counts as an absent key (schema §1.2).
    fn ask<T: FromNode>(&mut self, key: &'a str) -> Option<&'a Node> {
        self.asked.push(key);
        self.entries
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| &entry.value)
            .filter(|node| !T::is_empty(node))
    }

    fn read<T: FromNode>(&self, key: &str, node: &Node) -> Result<T, Reported> {
        self.problems
            .adopt(T::from_node(node, &self.cursor.key(key)))
    }

    fn missing(&self, key: &str, message: &str) -> Reported {
        let line = self
            .entries
            .iter()
            .find(|entry| entry.key == key)
            .map_or(self.line, |entry| entry.value.line);
        self.problems
            .record(self.cursor.key(key).problem(line, message));
        Reported
    }
}

/// Reads a nested map, such as one Source, with its own collector.
pub(crate) fn read_map<T>(
    node: &Node,
    cursor: &Cursor<'_>,
    read: impl FnOnce(&mut Fields<'_>) -> Result<T, Reported>,
) -> Result<T, Vec<Problem>> {
    let problems = Problems::default();
    let value = Fields::new(node, cursor.clone(), &problems).and_then(|mut fields| {
        let value = read(&mut fields);
        fields.finish();
        value
    });
    problems.into_result(value)
}

#[cfg(test)]
mod tests {
    use kariera_domain::{Text, Translations};

    use super::{Fields, Presence, read_map};
    use crate::cursor::Cursor;
    use crate::names::Names;
    use crate::problems::{Problems, Reported};
    use crate::yaml::{self, Node};
    use crate::{Location, Problem};

    fn node(text: &str) -> Node {
        yaml::parse(text, 1).unwrap().unwrap()
    }

    fn problem(field: &str, line: usize, message: &str) -> Problem {
        Problem::new(Location::Field(field.to_owned()), Some(line), message)
    }

    fn read<T>(
        text: &str,
        read: impl FnOnce(&mut Fields<'_>) -> Result<T, Reported>,
    ) -> Result<T, Vec<Problem>> {
        read_map(&node(text), &Cursor::root(&Names::default()), read)
    }

    fn text(value: &str) -> Text {
        value.parse().unwrap()
    }

    #[test]
    fn a_required_field_is_read() {
        let name = read("name: Acme\n", |fields| fields.required::<Text>("name"));

        assert_eq!(name, Ok(text("Acme")));
    }

    #[test]
    fn a_missing_required_field_is_reported_at_the_map() {
        let name = read("other: x\n", |fields| {
            fields.skip("other");
            fields.required::<Text>("name")
        });

        assert_eq!(name, Err(vec![problem("name", 1, "is required")]));
    }

    #[test]
    fn an_empty_required_field_is_reported_at_its_line() {
        let name = read("a: 1\nname:\n", |fields| {
            fields.skip("a");
            fields.required::<Text>("name")
        });

        assert_eq!(name, Err(vec![problem("name", 2, "is required")]));
    }

    #[test]
    fn an_optional_field_may_be_empty_or_absent() {
        let both = read("location: ~\n", |fields| {
            let location = fields.optional::<Text>("location");
            let website = fields.optional::<Text>("website");
            Ok((location?, website?))
        });

        assert_eq!(both, Ok((None, None)));
    }

    #[test]
    fn a_bad_value_is_reported_with_its_field() {
        let reviewed = read("reviewed: yes\n", |fields| {
            fields.required::<bool>("reviewed")
        });

        assert_eq!(
            reviewed,
            Err(vec![problem("reviewed", 1, "must be true or false")])
        );
    }

    #[test]
    fn every_bad_field_is_reported() {
        let both = read("a: yes\nb: no\n", |fields| {
            let first = fields.required::<bool>("a");
            let second = fields.required::<bool>("b");
            Ok((first?, second?))
        });

        assert_eq!(
            both,
            Err(vec![
                problem("a", 1, "must be true or false"),
                problem("b", 2, "must be true or false"),
            ])
        );
    }

    #[test]
    fn an_unknown_key_is_reported() {
        let name = read("name: Acme\ncolour: red\n", |fields| {
            fields.required::<Text>("name")
        });

        assert_eq!(
            name,
            Err(vec![problem(
                "colour",
                2,
                "is not a field the schema allows here"
            )])
        );
    }

    #[test]
    fn an_absent_list_is_empty() {
        let aliases = read("{}", |fields| fields.list::<Text>("aliases"));

        assert_eq!(aliases, Ok(Vec::new()));
    }

    #[test]
    fn absent_translations_are_none() {
        let labels = read("{}", |fields| fields.translations::<Text>("labels"));

        assert_eq!(labels, Ok(Translations::none()));
    }

    #[test]
    fn a_conditional_field_follows_its_presence() {
        let required = Presence::Required("kind is invoice".to_owned());
        let empty = Presence::Empty("kind is recalled".to_owned());

        let set = read("path: a.pdf\n", |fields| {
            fields.conditional::<Text>("path", Ok(required.clone()))
        });
        let missing = read("{}", |fields| {
            fields.conditional::<Text>("path", Ok(required))
        });
        let forbidden = read("path: a.pdf\n", |fields| {
            fields.conditional::<Text>("path", Ok(empty.clone()))
        });
        let left_empty = read("{}", |fields| fields.conditional::<Text>("path", Ok(empty)));
        let optional = read("{}", |fields| {
            fields.conditional::<Text>("path", Ok(Presence::Optional))
        });

        assert_eq!(set, Ok(Some(text("a.pdf"))));
        assert_eq!(
            missing,
            Err(vec![problem(
                "path",
                1,
                "is required because kind is invoice"
            )])
        );
        assert_eq!(
            forbidden,
            Err(vec![problem(
                "path",
                1,
                "must be empty because kind is recalled"
            )])
        );
        assert_eq!(left_empty, Ok(None));
        assert_eq!(optional, Ok(None));
    }

    #[test]
    fn a_conditional_field_without_a_presence_is_not_checked() {
        let path = read("path: a.pdf\n", |fields| {
            fields.conditional::<Text>("path", Err(Reported))
        });

        assert_eq!(path, Err(Vec::new()));
    }

    #[test]
    fn a_list_or_scalar_is_not_a_map_of_fields() {
        let problems = Problems::default();
        let names = Names::default();

        let list = node("- a\n");
        let fields = Fields::new(&list, Cursor::root(&names).key("role"), &problems);

        assert!(fields.is_err());
        assert_eq!(
            problems.into_result(Ok(())),
            Err(vec![problem("role", 1, "must be a map of fields")])
        );
    }
}
