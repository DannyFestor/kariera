use crate::names::Names;
use crate::{Location, Problem};

/// Where a reader is in a note's frontmatter, as a field path such as `sources[1].kind`, and the vault's note
/// names that links resolve against.
#[derive(Debug, Clone)]
pub(crate) struct Cursor<'names> {
    path: String,
    names: &'names Names,
}

impl<'names> Cursor<'names> {
    pub(crate) const fn root(names: &'names Names) -> Self {
        Self {
            path: String::new(),
            names,
        }
    }

    pub(crate) fn key(&self, key: &str) -> Self {
        let path = if self.path.is_empty() {
            key.to_owned()
        } else {
            format!("{}.{key}", self.path)
        };
        Self { path, ..*self }
    }

    pub(crate) fn item(&self, index: usize) -> Self {
        Self {
            path: format!("{}[{index}]", self.path),
            ..*self
        }
    }

    pub(crate) const fn names(&self) -> &'names Names {
        self.names
    }

    pub(crate) fn problem(&self, line: usize, message: impl Into<String>) -> Problem {
        Problem::new(Location::Field(self.path.clone()), Some(line), message)
    }
}

#[cfg(test)]
mod tests {
    use super::Cursor;
    use crate::names::Names;
    use crate::{Location, Problem};

    #[test]
    fn keys_and_items_build_a_field_path() {
        let names = Names::default();
        let cursor = Cursor::root(&names).key("sources").item(1).key("kind");

        assert_eq!(
            cursor.problem(9, "is required"),
            Problem::new(
                Location::Field("sources[1].kind".to_owned()),
                Some(9),
                "is required"
            )
        );
    }
}
