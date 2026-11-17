use std::fmt;

use kariera_vault::{Node, Value};

/// A place in a note's frontmatter, written as the vault reader writes it: `sources[1].path`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FieldPath(Vec<Step>);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Step {
    Key(String),
    Index(usize),
}

impl FieldPath {
    pub(crate) fn of(key: &str) -> Self {
        Self(vec![Step::Key(key.to_owned())])
    }

    #[must_use]
    pub(crate) fn key(mut self, key: &str) -> Self {
        self.0.push(Step::Key(key.to_owned()));
        self
    }

    #[must_use]
    pub(crate) fn item(mut self, index: usize) -> Self {
        self.0.push(Step::Index(index));
        self
    }

    /// The line of the value this path leads to, or of the deepest value that exists: a missing field is reported
    /// where it should have been written.
    pub(crate) fn line_in(&self, frontmatter: &Node) -> usize {
        let mut node = frontmatter;
        for step in &self.0 {
            match step.child_of(node) {
                Some(child) => node = child,
                None => break,
            }
        }
        node.line
    }
}

impl Step {
    fn child_of<'a>(&self, node: &'a Node) -> Option<&'a Node> {
        match (self, &node.value) {
            (Self::Key(key), Value::Mapping(_)) => node.get(key),
            (Self::Index(index), Value::Sequence(items)) => items.get(*index),
            (Self::Key(_), Value::Scalar { .. } | Value::Sequence(_))
            | (Self::Index(_), Value::Scalar { .. } | Value::Mapping(_)) => None,
        }
    }
}

impl fmt::Display for FieldPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (position, step) in self.0.iter().enumerate() {
            match step {
                Step::Key(key) if position == 0 => write!(formatter, "{key}")?,
                Step::Key(key) => write!(formatter, ".{key}")?,
                Step::Index(index) => write!(formatter, "[{index}]")?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kariera_vault::Vault;

    use super::FieldPath;

    fn frontmatter() -> kariera_vault::Node {
        let vault =
            Vault::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/vault"))
                .expect("the fixture vault loads");
        vault
            .notes()
            .iter()
            .find(|note| note.name.as_str() == "2023-10 Globex")
            .map(|note| note.frontmatter.clone())
            .expect("the fixture has the note")
    }

    #[test]
    fn a_path_is_written_like_the_readers() {
        let path = FieldPath::of("sources").item(1).key("note");

        assert_eq!(path.to_string(), "sources[1].note");
        assert_eq!(FieldPath::of("end").to_string(), "end");
    }

    #[test]
    fn a_path_finds_the_line_of_its_value() {
        let frontmatter = frontmatter();

        assert_eq!(FieldPath::of("client").line_in(&frontmatter), 3);
        assert_eq!(
            FieldPath::of("sources")
                .item(1)
                .key("note")
                .line_in(&frontmatter),
            19
        );
    }

    #[test]
    fn a_missing_field_is_reported_at_the_deepest_value_that_exists() {
        let frontmatter = frontmatter();

        assert_eq!(FieldPath::of("end").line_in(&frontmatter), 2);
        assert_eq!(
            FieldPath::of("sources")
                .item(0)
                .key("nothing")
                .line_in(&frontmatter),
            16
        );
    }
}
