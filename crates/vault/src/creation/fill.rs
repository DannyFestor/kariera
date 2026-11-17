use std::fmt::Display;

use kariera_domain::{NoteName, Text};

/// A field of the template's frontmatter, as a dotted path (`title.en`), and the YAML to write after its colon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Fill {
    pub(crate) field: &'static str,
    pub(crate) value: String,
}

/// Words YAML reads as something other than a string when written plain.
const RESERVED_WORDS: [&str; 9] = ["true", "false", "null", "yes", "no", "on", "off", "y", "n"];

impl Fill {
    /// For values whose written form is always plain YAML: months and slugs.
    pub(crate) fn plain(field: &'static str, value: impl Display) -> Self {
        Self {
            field,
            value: value.to_string(),
        }
    }

    /// Links are always double-quoted (schema §1.4).
    pub(crate) fn link(field: &'static str, target: &NoteName) -> Self {
        Self {
            field,
            value: format!("\"[[{target}]]\""),
        }
    }

    /// Plain unless YAML would read it as another type or as syntax (schema §1.2). A note name holds none of the
    /// characters that are special inside a plain string, so only its first character and reserved words matter.
    pub(crate) fn text(field: &'static str, text: &Text) -> Self {
        let text = text.as_str();
        let is_plain = text.starts_with(char::is_alphabetic)
            && !RESERVED_WORDS.contains(&text.to_lowercase().as_str());
        let value = if is_plain {
            text.to_owned()
        } else {
            format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
        };
        Self { field, value }
    }
}

/// Writes each fill after the colon of its empty frontmatter field. Fails with the first field the template does
/// not have as an empty field, so a template that drifted from the schema is reported, not half filled.
pub(crate) fn fill_template(template: &str, fills: &[Fill]) -> Result<String, String> {
    let mut lines: Vec<String> = template.split_inclusive('\n').map(str::to_owned).collect();
    for fill in fills {
        fill_field(&mut lines, fill).ok_or_else(|| fill.field.to_owned())?;
    }
    Ok(lines.concat())
}

fn fill_field(lines: &mut [String], fill: &Fill) -> Option<()> {
    let (parent, indent, key) = match fill.field.split_once('.') {
        Some((parent, child)) => (Some(parent), "  ", child),
        None => (None, "", fill.field),
    };
    let (opening, rest) = lines.split_first_mut()?;
    if !is_delimiter(opening) {
        return None;
    }
    let mut inside_parent = parent.is_none();
    for line in rest.iter_mut().take_while(|line| !is_delimiter(line)) {
        if let Some(parent) = parent {
            if is_empty_key(line, "", parent) {
                inside_parent = true;
                continue;
            }
            inside_parent &= line.starts_with(' ');
        }
        if inside_parent && is_empty_key(line, indent, key) {
            return write_value(line, &fill.value);
        }
    }
    None
}

fn is_delimiter(line: &str) -> bool {
    line.trim_end() == "---"
}

fn is_empty_key(line: &str, indent: &str, key: &str) -> bool {
    line.trim_end() == format!("{indent}{key}:")
}

fn write_value(line: &mut String, value: &str) -> Option<()> {
    let content = line.trim_end();
    let line_ending = line.strip_prefix(content)?;
    *line = format!("{content} {value}{line_ending}");
    Some(())
}

#[cfg(test)]
mod tests {
    use super::{Fill, fill_template};

    const TEMPLATE: &str = "---\ntype: project\nengagement:\ntitle:\n  en:\n  ja:\nreviewed: false\n---\n\n## Description\n\nengagement:\n";

    fn fill(field: &'static str, value: &str) -> Fill {
        Fill::plain(field, value)
    }

    #[test]
    fn a_top_level_field_gets_its_value_after_the_colon() {
        let filled = fill_template(TEMPLATE, &[fill("engagement", "x")]).unwrap();

        assert_eq!(
            filled,
            TEMPLATE.replacen("engagement:\n", "engagement: x\n", 1)
        );
    }

    #[test]
    fn a_nested_field_is_found_under_its_parent_only() {
        let filled = fill_template(TEMPLATE, &[fill("title.ja", "y")]).unwrap();

        assert_eq!(filled, TEMPLATE.replacen("  ja:\n", "  ja: y\n", 1));
    }

    #[test]
    fn a_nested_field_of_another_parent_is_not_found() {
        let template = "---\ntitle:\nrole:\n  en:\n---\n";

        assert_eq!(
            fill_template(template, &[fill("title.en", "x")]),
            Err("title.en".to_owned())
        );
    }

    #[test]
    fn the_body_is_not_searched() {
        let template = "---\ntype: project\n---\nengagement:\n";

        assert_eq!(
            fill_template(template, &[fill("engagement", "x")]),
            Err("engagement".to_owned())
        );
    }

    #[test]
    fn a_field_that_already_has_a_value_is_not_filled() {
        assert_eq!(
            fill_template(TEMPLATE, &[fill("reviewed", "true")]),
            Err("reviewed".to_owned())
        );
    }

    #[test]
    fn text_without_frontmatter_is_not_filled() {
        assert_eq!(
            fill_template("engagement:\n", &[fill("engagement", "x")]),
            Err("engagement".to_owned())
        );
    }

    #[test]
    fn a_missing_field_is_reported_by_its_path() {
        assert_eq!(
            fill_template(TEMPLATE, &[fill("start", "x")]),
            Err("start".to_owned())
        );
    }

    #[test]
    fn a_field_on_the_last_line_without_a_line_ending_is_filled() {
        assert_eq!(
            fill_template("---\nstart:\n---", &[fill("start", "x")]),
            Ok("---\nstart: x\n---".to_owned())
        );
    }

    #[test]
    fn text_is_quoted_when_yaml_would_not_read_a_string() {
        let quoted = |text: &str| Fill::text("title.en", &text.parse().unwrap()).value;

        assert_eq!(quoted("Billing"), "Billing");
        assert_eq!(quoted("Ünïcode"), "Ünïcode");
        assert_eq!(quoted("NO"), "\"NO\"");
        assert_eq!(quoted("-dash"), "\"-dash\"");
        assert_eq!(quoted("3D Viewer"), "\"3D Viewer\"");
        assert_eq!(quoted("-say \"hi\""), "\"-say \\\"hi\\\"\"");
        assert_eq!(quoted("-back\\slash"), "\"-back\\\\slash\"");
    }
}
