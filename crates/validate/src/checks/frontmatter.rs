//! Schema §1.2: fields in the order of the type's table, block style, and quotes only where they are required.

use kariera_vault::{Entry, Node, NoteFile, Style, Value, requires_quotes};

use crate::context::Context;
use crate::field_path::FieldPath;
use crate::finding::{At, Finding, Rule};
use crate::layout::{Field, Shape, fields_of};

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    context
        .files()
        .iter()
        .flat_map(|file| {
            let mut walk = Walk {
                file,
                findings: Vec::new(),
            };
            if let Value::Mapping(entries) = &file.frontmatter.value {
                walk.map(entries, fields_of(file.note_type), None);
            }
            walk.findings
        })
        .collect()
}

struct Walk<'a> {
    file: &'a NoteFile,
    findings: Vec<Finding>,
}

impl Walk<'_> {
    fn report(&mut self, rule: Rule, path: FieldPath, message: impl Into<String>) {
        self.findings
            .push(Finding::new(self.file, rule, At::Field(path), message));
    }

    fn node(&mut self, node: &Node, shape: &Shape, path: &FieldPath) {
        if node.flow {
            self.report(
                Rule::YamlStyle,
                path.clone(),
                "must be written in block style, not as `[…]` or `{…}`",
            );
            return;
        }
        // A key without a value or with `null` is empty (schema §1.2), which has no style.
        if node.is_empty() {
            return;
        }
        match (shape, &node.value) {
            (Shape::Text, Value::Scalar { text, style }) => self.text(text, *style, path),
            (Shape::Unquoted, Value::Scalar { style, .. }) => self.unquoted(*style, path),
            (Shape::Link, Value::Scalar { style, .. }) => self.link(*style, path),
            (Shape::List(item), Value::Sequence(items)) => self.list(items, item, path),
            (Shape::Map(fields), Value::Mapping(entries)) => self.map(entries, fields, Some(path)),
            (Shape::LinkKeyed(value), Value::Mapping(entries)) => {
                self.link_keyed(entries, value, path);
            }
            (Shape::Free, _)
            | (
                Shape::Text | Shape::Unquoted | Shape::Link,
                Value::Sequence(_) | Value::Mapping(_),
            )
            | (Shape::List(_), Value::Scalar { .. } | Value::Mapping(_))
            | (Shape::Map(_) | Shape::LinkKeyed(_), Value::Scalar { .. } | Value::Sequence(_)) => {}
        }
    }

    fn text(&mut self, text: &str, style: Style, path: &FieldPath) {
        match style {
            Style::Plain if requires_quotes(text) => self.report(
                Rule::YamlStyle,
                path.clone(),
                format!("must be quoted: YAML reads {text} as something other than text"),
            ),
            Style::SingleQuoted | Style::DoubleQuoted if !requires_quotes(text) => self.report(
                Rule::YamlStyle,
                path.clone(),
                "must be plain: YAML does not require quotes here",
            ),
            Style::Block => self.report(
                Rule::YamlStyle,
                path.clone(),
                "must be written on one line, not as a `|` or `>` block",
            ),
            Style::Plain | Style::SingleQuoted | Style::DoubleQuoted => {}
        }
    }

    fn unquoted(&mut self, style: Style, path: &FieldPath) {
        if style != Style::Plain {
            self.report(
                Rule::YamlStyle,
                path.clone(),
                "must be written plain, without quotes",
            );
        }
    }

    fn link(&mut self, style: Style, path: &FieldPath) {
        if style != Style::DoubleQuoted {
            self.report(
                Rule::YamlStyle,
                path.clone(),
                "must be a link in double quotes, such as \"[[Note name]]\"",
            );
        }
    }

    fn list(&mut self, items: &[Node], item: &Shape, path: &FieldPath) {
        for (index, node) in items.iter().enumerate() {
            self.node(node, item, &path.clone().item(index));
        }
    }

    fn link_keyed(&mut self, entries: &[Entry], value: &Shape, path: &FieldPath) {
        for entry in entries {
            let entry_path = path.clone().key(&entry.key);
            if entry.key_style != Style::DoubleQuoted {
                self.report(
                    Rule::YamlStyle,
                    entry_path.clone(),
                    "must be a link in double quotes, such as \"[[Note name]]\"",
                );
            }
            self.node(&entry.value, value, &entry_path);
        }
    }

    /// `path` is `None` for the frontmatter itself.
    fn map(&mut self, entries: &[Entry], fields: &[Field], path: Option<&FieldPath>) {
        self.entries_in_order(entries, fields, path);
        self.entries(entries, fields, path);
    }

    fn entries(&mut self, entries: &[Entry], fields: &[Field], path: Option<&FieldPath>) {
        for entry in entries {
            if let Some(field) = fields.iter().find(|field| field.key == entry.key) {
                self.node(&entry.value, &field.shape, &entry_path(path, &entry.key));
            }
        }
    }

    fn entries_in_order(&mut self, entries: &[Entry], fields: &[Field], path: Option<&FieldPath>) {
        let mut latest: Option<usize> = None;
        for entry in entries {
            let Some(position) = fields.iter().position(|field| field.key == entry.key) else {
                continue;
            };
            match latest {
                Some(seen) if position.cmp(&seen).is_lt() => {
                    let after = fields.get(seen).map_or("", |field| field.key);
                    self.report(
                        Rule::FieldOrder,
                        entry_path(path, &entry.key),
                        format!("must come before `{after}`: fields follow the order of the schema's table"),
                    );
                }
                Some(_) | None => latest = Some(position),
            }
        }
    }
}

fn entry_path(parent: Option<&FieldPath>, key: &str) -> FieldPath {
    parent.map_or_else(|| FieldPath::of(key), |parent| parent.clone().key(key))
}

#[cfg(test)]
mod tests {
    use crate::test_vault::TestVault;

    const GLOBEX: &str = "Organizations/Globex.md";
    const ACME: &str = "Organizations/Acme.md";

    fn findings_of(test: &str, note: &str, from: &str, to: &str) -> Vec<String> {
        let vault = TestVault::new(test);
        vault.replace(note, from, to);
        vault.lines()
    }

    #[test]
    fn fields_follow_the_order_of_the_schema_table() {
        let lines = findings_of(
            "style_order",
            GLOBEX,
            "kind:\n  - client\nreviewed: false",
            "reviewed: false\nkind:\n  - client",
        );

        assert_eq!(
            lines,
            [
                "Organizations/Globex.md:6: error [field-order] field `kind` must come before `reviewed`: fields follow the order of the schema's table"
            ]
        );
    }

    #[test]
    fn type_comes_first() {
        let lines = findings_of(
            "style_type_first",
            GLOBEX,
            "type: organization\nname: Globex",
            "name: Globex\ntype: organization",
        );

        assert_eq!(
            lines,
            [
                "Organizations/Globex.md:3: error [field-order] field `type` must come before `name`: fields follow the order of the schema's table"
            ]
        );
    }

    #[test]
    fn every_field_out_of_place_is_reported() {
        let lines = findings_of(
            "style_order_many",
            ACME,
            "location: Shibuya, Tokyo\nwebsite: https://acme.example\n",
            "website: https://acme.example\nlocation: Shibuya, Tokyo\n",
        );

        assert_eq!(lines.len(), 1);
        assert!(
            lines[0].contains("field `location` must come before `website`"),
            "{lines:?}"
        );
    }

    #[test]
    fn the_keys_of_a_translatable_map_follow_en_ja_de() {
        let lines = findings_of(
            "style_map3_order",
            "Engagements/2019-04 Müller Logistik.md",
            "  en: Software developer\n  ja: ソフトウェア開発者\n",
            "  ja: ソフトウェア開発者\n  en: Software developer\n",
        );

        assert_eq!(
            lines,
            [
                "Engagements/2019-04 Müller Logistik.md:7: error [field-order] field `role.en` must come before `ja`: fields follow the order of the schema's table"
            ]
        );
    }

    #[test]
    fn the_keys_of_a_source_follow_kind_path_note() {
        let lines = findings_of(
            "style_source_order",
            "Skills/TypeScript.md",
            "  - kind: skill-sheet\n    path: .scratch/_inputs/applications/skill-sheet-2023.xlsx\n",
            "  - path: .scratch/_inputs/applications/skill-sheet-2023.xlsx\n    kind: skill-sheet\n",
        );

        assert_eq!(
            lines,
            [
                "Skills/TypeScript.md:13: error [field-order] field `sources[0].kind` must come before `path`: fields follow the order of the schema's table"
            ]
        );
    }

    #[test]
    fn the_keys_of_a_rate_step_follow_from_rate() {
        let lines = findings_of(
            "style_rate_step_order",
            "Contracts/2021-04 Acme Contract.md",
            "  - from: 2021-04\n    rate: 4500\n",
            "  - rate: 4500\n    from: 2021-04\n",
        );

        assert_eq!(
            lines,
            [
                "Contracts/2021-04 Acme Contract.md:11: error [field-order] field `rate_history[0].from` must come before `rate`: fields follow the order of the schema's table"
            ]
        );
    }

    #[test]
    fn a_list_in_flow_style_is_reported() {
        let lines = findings_of(
            "style_flow_list",
            "Skills/Go.md",
            "aliases:\n  - Golang\n",
            "aliases: [Golang]\n",
        );

        assert_eq!(
            lines,
            [
                "Skills/Go.md:4: error [yaml-style] field `aliases` must be written in block style, not as `[…]` or `{…}`"
            ]
        );
    }

    #[test]
    fn an_empty_flow_collection_is_still_flow_style() {
        let lines = findings_of(
            "style_flow_empty",
            GLOBEX,
            "kind:\n  - client\n",
            "kind:\n  - client\naliases: []\nlocation: {}\n",
        );
        let order = lines
            .iter()
            .filter(|line| line.contains("[field-order]"))
            .count();
        let flow = lines
            .iter()
            .filter(|line| line.contains("block style"))
            .count();

        assert_eq!((order, flow), (1, 2), "{lines:?}");
    }

    #[test]
    fn a_map_in_flow_style_is_reported() {
        let lines = findings_of(
            "style_flow_map",
            "Skills/Japanese.md",
            "labels:\n  ja: 日本語\n  de: Japanisch\n",
            "labels: {ja: 日本語, de: Japanisch}\n",
        );

        assert!(
            lines
                .iter()
                .any(|line| line.contains("field `labels` must be written in block style")),
            "{lines:?}"
        );
    }

    #[test]
    fn a_link_is_written_in_double_quotes() {
        let lines = findings_of(
            "style_link_quotes",
            "Engagements/2021-04 Acme.md",
            "client: \"[[Acme]]\"",
            "client: '[[Acme]]'",
        );

        assert_eq!(
            lines,
            [
                "Engagements/2021-04 Acme.md:3: error [yaml-style] field `client` must be a link in double quotes, such as \"[[Note name]]\""
            ]
        );
    }

    #[test]
    fn a_link_in_a_list_is_written_in_double_quotes() {
        let lines = findings_of(
            "style_link_list",
            "Projects/Acme - Ledger.md",
            "  - \"[[TypeScript]]\"",
            "  - '[[TypeScript]]'",
        );

        assert_eq!(
            lines,
            [
                "Projects/Acme - Ledger.md:23: error [yaml-style] field `skills[0]` must be a link in double quotes, such as \"[[Note name]]\""
            ]
        );
    }

    #[test]
    fn the_key_of_project_hours_is_a_double_quoted_link() {
        let lines = findings_of(
            "style_link_key",
            "Monthly Logs/2025/2025-07 Globex Log.md",
            "  \"[[Globex - Search]]\": 100",
            "  '[[Globex - Search]]': 100",
        );

        assert_eq!(
            lines,
            [
                "Monthly Logs/2025/2025-07 Globex Log.md:7: error [yaml-style] field `project_hours.[[Globex - Search]]` must be a link in double quotes, such as \"[[Note name]]\""
            ]
        );
    }

    /// A note, a value as written, the same value quoted, and the field the finding names.
    const QUOTED_UNQUOTED_VALUES: [(&str, &str, &str, &str); 6] = [
        (
            "Engagements/2018-07 Initech.md",
            "start: 2018-07",
            "start: \"2018-07\"",
            "start",
        ),
        (
            "Contracts/2025-04 Globex Contract.md",
            "start: 2025-04-01",
            "start: '2025-04-01'",
            "start",
        ),
        (
            "Contracts/2025-04 Globex Contract.md",
            "rate: 640000",
            "rate: \"640000\"",
            "rate",
        ),
        (
            "Projects/Acme - Shop.md",
            "confidential: false",
            "confidential: \"false\"",
            "confidential",
        ),
        (
            "Skills/Go.md",
            "category: language",
            "category: \"language\"",
            "category",
        ),
        (
            "Projects/Acme - Shop.md",
            "  - basic-design",
            "  - \"basic-design\"",
            "phases[0]",
        ),
    ];

    #[test]
    fn months_dates_numbers_booleans_and_enums_are_written_without_quotes() {
        for (note, from, to, field) in QUOTED_UNQUOTED_VALUES {
            let lines = findings_of("style_unquoted", note, from, to);

            assert_eq!(lines.len(), 1, "{lines:?}");
            assert!(
                lines[0].contains(&format!(
                    "field `{field}` must be written plain, without quotes"
                )),
                "{lines:?}"
            );
        }
    }

    #[test]
    fn text_is_written_plain_unless_yaml_requires_quotes() {
        for to in ["location: \"Shibuya, Tokyo\"", "location: 'Shibuya, Tokyo'"] {
            let lines = findings_of("style_text_quotes", ACME, "location: Shibuya, Tokyo", to);

            assert_eq!(
                lines,
                [
                    "Organizations/Acme.md:10: error [yaml-style] field `location` must be plain: YAML does not require quotes here"
                ]
            );
        }
    }

    #[test]
    fn text_that_yaml_would_read_as_something_else_is_quoted() {
        for text in ["2", "true", "1.5", "0x1F", "null"] {
            let lines = findings_of(
                "style_text_plain",
                "Qualifications/Example Japanese Test B2.md",
                "level: B2",
                &format!("level: {text}"),
            );

            if text == "null" {
                assert!(
                    lines.iter().all(|line| !line.contains("yaml-style")),
                    "{lines:?}"
                );
            } else {
                assert_eq!(
                    lines,
                    [format!(
                        "Qualifications/Example Japanese Test B2.md:9: error [yaml-style] field `level` must be quoted: YAML reads {text} as something other than text"
                    )]
                );
            }
        }
    }

    #[test]
    fn quotes_where_yaml_requires_them_are_fine() {
        let lines = findings_of(
            "style_required_quotes",
            "Qualifications/Example Japanese Test B2.md",
            "level: B2",
            "level: \"2\"",
        );

        assert_eq!(lines, Vec::<String>::new());
    }

    #[test]
    fn text_is_written_on_one_line_not_as_a_block() {
        let lines = findings_of(
            "style_block",
            ACME,
            "location: Shibuya, Tokyo",
            "location: >-\n  Shibuya, Tokyo",
        );

        assert_eq!(
            lines,
            [
                "Organizations/Acme.md:11: error [yaml-style] field `location` must be written on one line, not as a `|` or `>` block"
            ]
        );
    }

    #[test]
    fn empty_values_have_no_style() {
        let lines = findings_of(
            "style_empty",
            ACME,
            "location: Shibuya, Tokyo\nwebsite: https://acme.example\n",
            "location:\nwebsite: ~\n",
        );

        assert_eq!(lines, Vec::<String>::new());
    }

    #[test]
    fn the_fingerprints_of_translated_from_are_not_checked_here() {
        let lines = findings_of(
            "style_fingerprints",
            "Profile.md",
            "  ja: 7c8d9e0f1a2b\n  de: 7c8d9e0f1a2b",
            "  ja: 123456789012\n  de: \"7c8d9e0f1a2b\"",
        );

        assert_eq!(lines, Vec::<String>::new());
    }
}
