use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use kariera_domain::{NoteName, NoteType};

use crate::body::Body;
use crate::names::nfc;
use crate::value::{FromScalar, scalar_text};
use crate::yaml::{self, Node, Value};
use crate::{Location, Problem};

/// A note file read as far as it can be without the rest of the vault: its name, its type, its frontmatter as a
/// tree and its body as sections. What could not be read is `None` and explained in `problems`.
#[derive(Debug)]
pub(crate) struct RawNote {
    pub(crate) path: PathBuf,
    pub(crate) name: Option<NoteName>,
    pub(crate) text: String,
    pub(crate) frontmatter: Option<Node>,
    pub(crate) note_type: Option<NoteType>,
    pub(crate) body: Body,
    pub(crate) problems: Vec<Problem>,
}

/// The frontmatter's YAML starts on the line after the opening `---` (schema §1.2).
const FIRST_YAML_LINE: usize = 2;

struct Split<'text> {
    yaml: &'text str,
    body: &'text str,
    first_body_line: usize,
}

impl RawNote {
    /// `path` is relative to the vault root.
    pub(crate) fn read(path: PathBuf, bytes: Vec<u8>) -> Self {
        let mut problems = Vec::new();
        let name = keep(&mut problems, note_name(&path));
        let text = keep(&mut problems, text(bytes));
        let split = text
            .as_deref()
            .and_then(|text| keep(&mut problems, split(text)));
        let frontmatter = split
            .as_ref()
            .and_then(|split| keep(&mut problems, frontmatter(split.yaml)));
        let note_type = frontmatter
            .as_ref()
            .and_then(|frontmatter| keep(&mut problems, note_type(frontmatter)));
        let body = split.map_or_else(Body::default, |split| {
            Body::parse(split.body, split.first_body_line)
        });
        Self {
            path,
            name,
            text: text.unwrap_or_default(),
            frontmatter,
            note_type,
            body,
            problems,
        }
    }
}

fn keep<T>(problems: &mut Vec<Problem>, result: Result<T, Problem>) -> Option<T> {
    result.map_err(|problem| problems.push(problem)).ok()
}

fn note_name(path: &Path) -> Result<NoteName, Problem> {
    let problem = |message: String| Problem::new(Location::Note, None, message);
    let stem = path
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or_else(|| problem("must have a UTF-8 file name".to_owned()))?;
    nfc(stem)
        .parse()
        .map_err(|error| problem(format!("has a file name that {error}")))
}

/// Notes are UTF-8 with LF line endings (schema §1.1).
fn text(bytes: Vec<u8>) -> Result<String, Problem> {
    let text = String::from_utf8(bytes).map_err(|error| {
        let at = error.utf8_error().valid_up_to();
        Problem::new(
            Location::Note,
            None,
            format!("must be UTF-8 text, but byte {at} is not"),
        )
    })?;
    match text.find('\r') {
        Some(at) => {
            let line = text
                .get(..at)
                .map_or(1, |before| before.lines().count().max(1));
            Err(Problem::new(
                Location::Note,
                Some(line),
                "must use LF line endings, not CRLF",
            ))
        }
        None => Ok(text),
    }
}

fn split(text: &str) -> Result<Split<'_>, Problem> {
    let missing = || {
        Problem::new(
            Location::Frontmatter,
            Some(1),
            "must open the note: a `---` line, the fields, then another `---` line",
        )
    };
    let after_opening = text.strip_prefix("---\n").ok_or_else(missing)?;
    let mut yaml_length = 0;
    for (closing_line, line) in (FIRST_YAML_LINE..).zip(after_opening.split_inclusive('\n')) {
        if line.trim_end_matches('\n') == "---" {
            return Ok(Split {
                yaml: after_opening.get(..yaml_length).unwrap_or_default(),
                body: after_opening
                    .get(yaml_length + line.len()..)
                    .unwrap_or_default(),
                first_body_line: closing_line + 1,
            });
        }
        yaml_length += line.len();
    }
    Err(missing())
}

/// An empty frontmatter is a map without fields.
fn frontmatter(yaml: &str) -> Result<Node, Problem> {
    let root = yaml::parse(yaml, FIRST_YAML_LINE)
        .map_err(|error| Problem::new(Location::Frontmatter, Some(error.line), error.message))?;
    match root {
        Some(
            node @ Node {
                value: Value::Mapping(_),
                ..
            },
        ) => Ok(node),
        Some(node) => Err(Problem::new(
            Location::Frontmatter,
            Some(node.line),
            "must be a map of fields",
        )),
        None => Ok(Node {
            line: FIRST_YAML_LINE,
            value: Value::Mapping(Vec::new()),
        }),
    }
}

fn note_type(frontmatter: &Node) -> Result<NoteType, Problem> {
    let problem = |line: usize, message: String| {
        Problem::new(Location::Field("type".to_owned()), Some(line), message)
    };
    let node = frontmatter
        .get("type")
        .filter(|node| !node.is_empty())
        .ok_or_else(|| problem(frontmatter.line, "is required".to_owned()))?;
    scalar_text(node)
        .and_then(NoteType::from_scalar)
        .map_err(|message| problem(node.line, message))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kariera_domain::NoteType;

    use super::RawNote;
    use crate::body::Body;
    use crate::{Location, Problem};

    fn read(path: &str, text: &str) -> RawNote {
        RawNote::read(PathBuf::from(path), text.as_bytes().to_vec())
    }

    #[test]
    fn a_note_is_split_into_frontmatter_and_body() {
        let note = read("Skills/Go.md", "---\ntype: skill\nname: Go\n---\n## A\n");

        assert_eq!(note.problems, Vec::new());
        assert_eq!(
            note.name.map(|name| name.to_string()),
            Some("Go".to_owned())
        );
        assert_eq!(note.note_type, Some(NoteType::Skill));
        assert_eq!(note.frontmatter.map(|node| node.line), Some(2));
        assert_eq!(note.body, Body::parse("## A\n", 5));
        assert_eq!(note.text, "---\ntype: skill\nname: Go\n---\n## A\n");
    }

    #[test]
    fn a_file_name_in_nfd_becomes_an_nfc_name() {
        let note = read(
            "Organizations/Mu\u{308}ller.md",
            "---\ntype: organization\n---\n",
        );

        assert_eq!(
            note.name.map(|name| name.to_string()),
            Some("Müller".to_owned())
        );
    }

    #[test]
    fn a_file_name_that_is_no_note_name_is_reported() {
        let note = read("Skills/.hidden.md", "---\ntype: skill\n---\n");

        assert_eq!(note.name, None);
        assert_eq!(
            note.problems,
            vec![Problem::new(
                Location::Note,
                None,
                "has a file name that must not start or end with a space or a dot"
            )]
        );
    }

    #[test]
    fn a_note_must_be_utf8() {
        let note = RawNote::read(PathBuf::from("Skills/Go.md"), vec![b'-', 0xff]);

        assert_eq!(
            note.problems,
            vec![Problem::new(
                Location::Note,
                None,
                "must be UTF-8 text, but byte 1 is not"
            )]
        );
        assert_eq!(note.frontmatter, None);
    }

    #[test]
    fn a_note_must_use_lf_line_endings() {
        let note = read("Skills/Go.md", "---\ntype: skill\r\n---\n");

        assert_eq!(
            note.problems,
            vec![Problem::new(
                Location::Note,
                Some(2),
                "must use LF line endings, not CRLF"
            )]
        );
    }

    #[test]
    fn a_note_without_frontmatter_is_reported() {
        for text in [
            "type: skill\n",
            "---\ntype: skill\n",
            "",
            "--- \ntype: skill\n---\n",
        ] {
            let note = read("Skills/Go.md", text);

            assert_eq!(
                note.problems,
                vec![Problem::new(
                    Location::Frontmatter,
                    Some(1),
                    "must open the note: a `---` line, the fields, then another `---` line"
                )],
                "{text:?}"
            );
        }
    }

    #[test]
    fn invalid_yaml_is_reported_on_its_line() {
        let note = read("Skills/Go.md", "---\ntype: skill\ntype: skill\n---\n");

        assert_eq!(
            note.problems,
            vec![Problem::new(
                Location::Frontmatter,
                Some(3),
                "repeats the key `type`"
            )]
        );
    }

    #[test]
    fn frontmatter_must_be_a_map() {
        let note = read("Skills/Go.md", "---\n- skill\n---\n");

        assert_eq!(
            note.problems,
            vec![Problem::new(
                Location::Frontmatter,
                Some(2),
                "must be a map of fields"
            )]
        );
    }

    #[test]
    fn the_type_is_required_and_must_be_a_note_type() {
        let empty = read("Skills/Go.md", "---\n---\n");
        let unknown = read("Skills/Go.md", "---\nname: Go\ntype: tool\n---\n");

        assert_eq!(
            empty.problems,
            vec![Problem::new(
                Location::Field("type".to_owned()),
                Some(2),
                "is required"
            )]
        );
        assert_eq!(
            unknown.problems,
            vec![Problem::new(
                Location::Field("type".to_owned()),
                Some(3),
                "must be one of organization, engagement, contract, project, monthly-log, skill, \
                 qualification, profile"
            )]
        );
    }

    #[test]
    fn a_body_starts_after_the_closing_line_even_at_the_end_of_the_file() {
        let note = read("Skills/Go.md", "---\ntype: skill\n---");

        assert_eq!(note.problems, Vec::new());
        assert_eq!(note.body, Body::default());
    }
}
