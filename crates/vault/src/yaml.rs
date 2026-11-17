//! The frontmatter as a tree that keeps what the schema's rules need: the order of keys, the line of every value,
//! how a scalar was written (plain, quoted, as a block), which tells `null` from the quoted text `"null"`, and
//! whether a collection was written in flow style.

use saphyr_parser::{Event, Parser, ScalarStyle, ScanError, Span, StrInput};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// 1-based, counted from the first line of the file.
    pub line: usize,
    pub value: Value,
    /// A list or map written in flow style, `[a, b]` or `{a: b}`. Never set for a scalar.
    pub flow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Scalar { text: String, style: Style },
    Sequence(Vec<Node>),
    Mapping(Vec<Entry>),
}

/// How a scalar is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Plain,
    SingleQuoted,
    DoubleQuoted,
    /// A `|` literal or a `>` folded block.
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub key: String,
    pub key_style: Style,
    pub value: Node,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SyntaxError {
    pub(crate) line: usize,
    pub(crate) message: String,
}

impl Node {
    /// Empty in the sense of schema §1.2: no value, `null`, `[]` or `{}`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match &self.value {
            Value::Scalar { text, style } => {
                *style == Style::Plain
                    && matches!(text.as_str(), "" | "~" | "null" | "Null" | "NULL")
            }
            Value::Sequence(items) => items.is_empty(),
            Value::Mapping(entries) => entries.is_empty(),
        }
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Self> {
        match &self.value {
            Value::Mapping(entries) => entries
                .iter()
                .find(|entry| entry.key == key)
                .map(|entry| &entry.value),
            Value::Scalar { .. } | Value::Sequence(_) => None,
        }
    }
}

/// `first_line` is the file line the YAML text starts on.
pub(crate) fn parse(text: &str, first_line: usize) -> Result<Option<Node>, SyntaxError> {
    let mut builder = Builder {
        events: Parser::new_from_str(text),
        characters: text.chars().collect(),
        lines_before: first_line.saturating_sub(1),
    };
    builder.document()
}

struct Builder<'input> {
    events: Parser<'input, StrInput<'input>>,
    /// The scanner reports positions in characters; the events do not say whether a collection is in flow style.
    characters: Vec<char>,
    lines_before: usize,
}

impl<'input> Builder<'input> {
    fn document(&mut self) -> Result<Option<Node>, SyntaxError> {
        self.next()?;
        let (start, _) = self.next()?;
        if matches!(start, Event::StreamEnd) {
            return Ok(None);
        }
        let (event, span) = self.next()?;
        let root = self.node(event, span)?;
        self.next()?;
        match self.next()? {
            (Event::StreamEnd, _) => Ok(Some(root)),
            (_, second_document) => {
                Err(self.error(second_document, "must hold one YAML document only"))
            }
        }
    }

    fn node(&mut self, event: Event<'input>, span: Span) -> Result<Node, SyntaxError> {
        let line = self.line(span);
        let value = match event {
            Event::Scalar(_, _, _, Some(_))
            | Event::SequenceStart(_, Some(_))
            | Event::MappingStart(_, Some(_)) => {
                return Err(self.error(span, "must not use YAML tags"));
            }
            Event::Alias(_) => return Err(self.error(span, "must not use YAML aliases")),
            Event::Scalar(text, style, _, None) => Value::Scalar {
                text: text.into_owned(),
                style: style_of(style),
            },
            Event::SequenceStart(_, None) => Value::Sequence(self.sequence()?),
            Event::MappingStart(_, None) => Value::Mapping(self.mapping()?),
            Event::Nothing
            | Event::StreamStart
            | Event::StreamEnd
            | Event::DocumentStart(_)
            | Event::DocumentEnd
            | Event::SequenceEnd
            | Event::MappingEnd => return Err(self.error(span, "is not valid YAML here")),
        };
        let flow = !matches!(value, Value::Scalar { .. }) && self.starts_flow_collection(span);
        Ok(Node { line, value, flow })
    }

    fn starts_flow_collection(&self, span: Span) -> bool {
        matches!(self.characters.get(span.start.index()), Some('[' | '{'))
    }

    fn sequence(&mut self) -> Result<Vec<Node>, SyntaxError> {
        let mut items = Vec::new();
        loop {
            let (event, span) = self.next()?;
            if matches!(event, Event::SequenceEnd) {
                return Ok(items);
            }
            items.push(self.node(event, span)?);
        }
    }

    fn mapping(&mut self) -> Result<Vec<Entry>, SyntaxError> {
        let mut entries: Vec<Entry> = Vec::new();
        loop {
            let (event, span) = self.next()?;
            if matches!(event, Event::MappingEnd) {
                return Ok(entries);
            }
            let (key, key_style) = self.key(event, span)?;
            if entries.iter().any(|entry| entry.key == key) {
                return Err(self.error(span, &format!("repeats the key `{key}`")));
            }
            let (value_event, value_span) = self.next()?;
            let value = self.node(value_event, value_span)?;
            entries.push(Entry {
                key,
                key_style,
                value,
            });
        }
    }

    fn key(&mut self, event: Event<'input>, span: Span) -> Result<(String, Style), SyntaxError> {
        match self.node(event, span)?.value {
            Value::Scalar { text, style } => Ok((text, style)),
            Value::Sequence(_) | Value::Mapping(_) => {
                Err(self.error(span, "must use text as keys, not lists or maps"))
            }
        }
    }

    fn next(&mut self) -> Result<(Event<'input>, Span), SyntaxError> {
        match self.events.next() {
            Some(Ok(event)) => Ok(event),
            Some(Err(error)) => Err(self.scan_error(&error)),
            None => Err(SyntaxError {
                line: self.lines_before,
                message: "ends in the middle of a value".to_owned(),
            }),
        }
    }

    fn line(&self, span: Span) -> usize {
        span.start.line() + self.lines_before
    }

    fn error(&self, span: Span, message: &str) -> SyntaxError {
        SyntaxError {
            line: self.line(span),
            message: message.to_owned(),
        }
    }

    fn scan_error(&self, error: &ScanError) -> SyntaxError {
        SyntaxError {
            line: error.marker().line() + self.lines_before,
            message: format!("is not valid YAML: {}", error.info()),
        }
    }
}

const fn style_of(style: ScalarStyle) -> Style {
    match style {
        ScalarStyle::Plain => Style::Plain,
        ScalarStyle::SingleQuoted => Style::SingleQuoted,
        ScalarStyle::DoubleQuoted => Style::DoubleQuoted,
        ScalarStyle::Literal | ScalarStyle::Folded => Style::Block,
    }
}

#[cfg(test)]
mod tests {
    use super::{Entry, Node, Style, SyntaxError, Value, parse};

    fn scalar(line: usize, text: &str, style: Style) -> Node {
        Node {
            line,
            value: Value::Scalar {
                text: text.to_owned(),
                style,
            },
            flow: false,
        }
    }

    fn root(text: &str) -> Node {
        parse(text, 2).unwrap().unwrap()
    }

    #[test]
    fn a_mapping_keeps_its_keys_in_order_with_file_lines() {
        let node = root("type: skill\nname: \"Go\"\n");

        assert_eq!(
            node,
            Node {
                line: 2,
                value: Value::Mapping(vec![
                    Entry {
                        key: "type".to_owned(),
                        key_style: Style::Plain,
                        value: scalar(2, "skill", Style::Plain),
                    },
                    Entry {
                        key: "name".to_owned(),
                        key_style: Style::Plain,
                        value: scalar(3, "Go", Style::DoubleQuoted),
                    },
                ]),
                flow: false,
            }
        );
    }

    #[test]
    fn a_block_sequence_becomes_a_list() {
        let node = root("kind:\n  - client\n  - agency\n");

        assert_eq!(
            node.get("kind"),
            Some(&Node {
                line: 3,
                value: Value::Sequence(vec![
                    scalar(3, "client", Style::Plain),
                    scalar(4, "agency", Style::Plain)
                ]),
                flow: false,
            })
        );
    }

    #[test]
    fn every_way_to_write_a_scalar_has_its_style() {
        let node = root("a: x\nb: 'x'\nc: \"x\"\nd: |-\n  x\ne: >-\n  x\n");

        let styles = ["a", "b", "c", "d", "e"].map(|key| match node.get(key).map(|n| &n.value) {
            Some(Value::Scalar { style, .. }) => Some(*style),
            _ => None,
        });

        assert_eq!(
            styles,
            [
                Some(Style::Plain),
                Some(Style::SingleQuoted),
                Some(Style::DoubleQuoted),
                Some(Style::Block),
                Some(Style::Block)
            ]
        );
    }

    #[test]
    fn a_key_keeps_its_style() {
        let node = root("\"[[A]]\": 1\nb: 2\n");

        let Value::Mapping(entries) = node.value else {
            panic!("not a map");
        };

        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.key_style)
                .collect::<Vec<_>>(),
            [Style::DoubleQuoted, Style::Plain]
        );
    }

    #[test]
    fn only_a_collection_written_in_flow_style_is_flow() {
        let node = root(
            "a: [x]\nb: {c: d}\ne: []\nf: {}\ng:\n  - h\ni:\n  j: k\nl: m\nn:\n  - [o]\n  - {p: q}\n",
        );

        let flow =
            ["a", "b", "e", "f", "g", "i", "l", "n"].map(|key| node.get(key).map(|node| node.flow));

        assert_eq!(
            flow,
            [
                Some(true),
                Some(true),
                Some(true),
                Some(true),
                Some(false),
                Some(false),
                Some(false),
                Some(false)
            ]
        );
        let Some(Value::Sequence(items)) = node.get("n").map(|node| &node.value) else {
            panic!("not a list");
        };
        assert_eq!(
            items.iter().map(|item| item.flow).collect::<Vec<_>>(),
            [true, true]
        );
    }

    #[test]
    fn plain_nulls_and_empty_collections_are_empty() {
        let node = root("a:\nb: ~\nc: null\nd: []\ne: {}\nf: \"\"\ng: \"null\"\nh: x\n");

        let empty =
            ["a", "b", "c", "d", "e", "f", "g", "h"].map(|key| node.get(key).unwrap().is_empty());

        assert_eq!(empty, [true, true, true, true, true, false, false, false]);
    }

    #[test]
    fn a_missing_key_or_a_non_mapping_has_no_value() {
        assert_eq!(root("a: 1\n").get("b"), None);
        assert_eq!(root("- a\n").get("a"), None);
    }

    #[test]
    fn an_empty_document_has_no_root() {
        assert_eq!(parse("", 2), Ok(None));
    }

    #[test]
    fn a_repeated_key_is_an_error_on_its_line() {
        let error = parse("name: a\nname: b\n", 2).unwrap_err();

        assert_eq!(
            error,
            SyntaxError {
                line: 3,
                message: "repeats the key `name`".to_owned(),
            }
        );
    }

    #[test]
    fn tags_aliases_and_complex_keys_are_refused() {
        for (text, message) in [
            ("a: !!str 5\n", "must not use YAML tags"),
            ("a: !x [1]\n", "must not use YAML tags"),
            ("a: &x 1\nb: *x\n", "must not use YAML aliases"),
            ("? [a]\n: 1\n", "must use text as keys, not lists or maps"),
        ] {
            assert_eq!(parse(text, 1).unwrap_err().message, message, "{text}");
        }
    }

    #[test]
    fn several_documents_are_refused() {
        let error = parse("a: 1\n---\nb: 2\n", 1).unwrap_err();

        assert_eq!(error.message, "must hold one YAML document only");
        assert_eq!(error.line, 2);
    }

    #[test]
    fn a_syntax_error_names_its_line() {
        let error = parse("a: 1\nb: [unclosed\n", 2).unwrap_err();

        assert!(
            error.message.starts_with("is not valid YAML: "),
            "{}",
            error.message
        );
        assert_eq!(
            error.line, 4,
            "the scanner stops at the end, where the ] is missing"
        );
    }
}
