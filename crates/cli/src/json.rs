//! The small part of JSON that `validate --json` writes: objects, arrays, strings, numbers and booleans.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Json {
    Null,
    Bool(bool),
    Number(usize),
    String(String),
    Array(Vec<Self>),
    Object(Vec<(&'static str, Self)>),
}

impl Json {
    pub(crate) fn string(text: impl Into<String>) -> Self {
        Self::String(text.into())
    }

    pub(crate) fn render(&self) -> String {
        match self {
            Self::Null => "null".to_owned(),
            Self::Bool(value) => value.to_string(),
            Self::Number(value) => value.to_string(),
            Self::String(text) => quoted(text),
            Self::Array(items) => {
                let items: Vec<String> = items.iter().map(Self::render).collect();
                format!("[{}]", items.join(","))
            }
            Self::Object(members) => {
                let members: Vec<String> = members
                    .iter()
                    .map(|(key, value)| format!("{}:{}", quoted(key), value.render()))
                    .collect();
                format!("{{{}}}", members.join(","))
            }
        }
    }
}

fn quoted(text: &str) -> String {
    let escaped: String = text.chars().map(escaped).collect();
    format!("\"{escaped}\"")
}

fn escaped(character: char) -> String {
    match character {
        '"' => "\\\"".to_owned(),
        '\\' => "\\\\".to_owned(),
        '\n' => "\\n".to_owned(),
        '\r' => "\\r".to_owned(),
        '\t' => "\\t".to_owned(),
        control if control.is_control() => format!("\\u{:04x}", u32::from(control)),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::Json;

    #[test]
    fn scalars_render_as_json_literals() {
        assert_eq!(Json::Null.render(), "null");
        assert_eq!(Json::Bool(true).render(), "true");
        assert_eq!(Json::Bool(false).render(), "false");
        assert_eq!(Json::Number(42).render(), "42");
    }

    #[test]
    fn a_string_escapes_what_json_requires() {
        let text = Json::string("a \"quote\", a \\ backslash,\na line,\ta tab,\r and \u{1}\u{1f}");

        assert_eq!(
            text.render(),
            r#""a \"quote\", a \\ backslash,\na line,\ta tab,\r and \u0001\u001f""#
        );
    }

    #[test]
    fn text_in_other_scripts_is_written_as_it_is() {
        assert_eq!(Json::string("日本語 Müller").render(), "\"日本語 Müller\"");
    }

    #[test]
    fn arrays_and_objects_nest_without_spaces() {
        let json = Json::Object(vec![
            ("empty", Json::Array(Vec::new())),
            (
                "items",
                Json::Array(vec![Json::Number(1), Json::Null, Json::string("x")]),
            ),
            ("inner", Json::Object(vec![("k", Json::Bool(true))])),
        ]);

        assert_eq!(
            json.render(),
            r#"{"empty":[],"items":[1,null,"x"],"inner":{"k":true}}"#
        );
    }

    #[test]
    fn object_keys_are_escaped_like_strings() {
        assert_eq!(
            Json::Object(vec![("a\"b", Json::Null)]).render(),
            r#"{"a\"b":null}"#
        );
    }
}
