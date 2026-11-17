//! The frontmatter as a tree that keeps what the schema's rules need: the order of keys, the line of every value,
//! and whether a scalar was written plain, which tells `null` from the quoted text `"null"`.

use saphyr_parser::{Event, Parser, ScalarStyle, ScanError, Span, StrInput};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Node {
    /// 1-based, counted from the first line of the file.
    pub(crate) line: usize,
    pub(crate) value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Value {
    Scalar { text: String, plain: bool },
    Sequence(Vec<Node>),
    Mapping(Vec<Entry>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) key: String,
    pub(crate) value: Node,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SyntaxError {
    pub(crate) line: usize,
    pub(crate) message: String,
}

impl Node {
    /// Empty in the sense of schema §1.2: no value, `null`, `[]` or `{}`.
    pub(crate) fn is_empty(&self) -> bool {
        match &self.value {
            Value::Scalar { text, plain } => {
                *plain && matches!(text.as_str(), "" | "~" | "null" | "Null" | "NULL")
            }
            Value::Sequence(items) => items.is_empty(),
            Value::Mapping(entries) => entries.is_empty(),
        }
    }

    pub(crate) fn get(&self, key: &str) -> Option<&Self> {
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
        lines_before: first_line.saturating_sub(1),
    };
    builder.document()
}

struct Builder<'input> {
    events: Parser<'input, StrInput<'input>>,
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
                plain: style == ScalarStyle::Plain,
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
        Ok(Node { line, value })
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
            let key = self.key(event, span)?;
            if entries.iter().any(|entry| entry.key == key) {
                return Err(self.error(span, &format!("repeats the key `{key}`")));
            }
            let (value_event, value_span) = self.next()?;
            let value = self.node(value_event, value_span)?;
            entries.push(Entry { key, value });
        }
    }

    fn key(&mut self, event: Event<'input>, span: Span) -> Result<String, SyntaxError> {
        match self.node(event, span)?.value {
            Value::Scalar { text, .. } => Ok(text),
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

#[cfg(test)]
mod tests {
    use super::{Entry, Node, SyntaxError, Value, parse};

    fn scalar(line: usize, text: &str, plain: bool) -> Node {
        Node {
            line,
            value: Value::Scalar {
                text: text.to_owned(),
                plain,
            },
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
                        value: scalar(2, "skill", true),
                    },
                    Entry {
                        key: "name".to_owned(),
                        value: scalar(3, "Go", false),
                    },
                ]),
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
                value: Value::Sequence(vec![scalar(3, "client", true), scalar(4, "agency", true)]),
            })
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
