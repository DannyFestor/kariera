//! Checks a JSON value against a JSON Schema (draft 2020-12) that uses only the keywords below. A schema with any
//! other keyword fails, so `schemas/career-export.schema.json` cannot grow a rule these tests would skip. A full
//! validator crate would bring in a licence outside the allowlist of `deny.toml` (boon) or some thirty crates
//! (jsonschema).

use regex::Regex;
use serde_json::{Map, Value};

/// Annotations: they describe and never constrain.
const ANNOTATIONS: [&str; 4] = ["$schema", "title", "description", "$defs"];

/// Every way `value` breaks `schema`, each with the JSON Pointer of the value; empty when it conforms.
pub(crate) fn violations(schema: &Value, value: &Value) -> Vec<String> {
    let mut checker = Checker {
        root: schema,
        found: Vec::new(),
    };
    checker.check(
        schema,
        &Instance {
            value,
            at: String::new(),
        },
    );
    checker.found
}

/// A value inside the checked document, with its JSON Pointer.
struct Instance<'value> {
    value: &'value Value,
    at: String,
}

impl Instance<'_> {
    fn child<'child>(&self, key: &str, value: &'child Value) -> Instance<'child> {
        Instance {
            value,
            at: format!("{}/{key}", self.at),
        }
    }
}

struct Checker<'schema> {
    root: &'schema Value,
    found: Vec<String>,
}

impl Checker<'_> {
    fn check(&mut self, schema: &Value, instance: &Instance<'_>) {
        let Some(keywords) = schema.as_object() else {
            self.report(instance, "has a schema that is not an object");
            return;
        };
        for keyword in keywords {
            if !ANNOTATIONS.contains(&keyword.0.as_str()) {
                self.keyword(keyword, keywords, instance);
            }
        }
    }

    fn keyword(
        &mut self,
        (keyword, rule): (&String, &Value),
        keywords: &Map<String, Value>,
        instance: &Instance<'_>,
    ) {
        let value = instance.value;
        match keyword.as_str() {
            "$ref" => self.reference(rule, instance),
            "type" => self.expect(
                is_of_type(rule, value),
                instance,
                &format!("is not of type {rule}"),
            ),
            "const" => self.expect(
                value == rule,
                instance,
                &format!("is not the constant {rule}"),
            ),
            "enum" => self.expect(
                is_listed(rule, value),
                instance,
                &format!("is not one of {rule}"),
            ),
            "minimum" => self.expect(
                is_at_least(rule, value),
                instance,
                &format!("is below {rule}"),
            ),
            "minLength" => self.expect(
                is_long_enough(rule, value),
                instance,
                &format!("is shorter than {rule}"),
            ),
            "pattern" => self.pattern(rule, instance),
            "required" => self.required(rule, instance),
            "properties" => self.properties(rule, instance),
            "additionalProperties" => self.additional_properties(rule, keywords, instance),
            "items" => self.items(rule, instance),
            other => self.report(instance, &format!("has the unsupported keyword {other:?}")),
        }
    }

    fn reference(&mut self, rule: &Value, instance: &Instance<'_>) {
        let root = self.root;
        let target = rule
            .as_str()
            .and_then(|pointer| pointer.strip_prefix('#'))
            .and_then(|pointer| root.pointer(pointer));
        match target {
            Some(schema) => self.check(schema, instance),
            None => self.report(instance, &format!("has a $ref {rule} that names nothing")),
        }
    }

    fn pattern(&mut self, rule: &Value, instance: &Instance<'_>) {
        let Some(text) = instance.value.as_str() else {
            return;
        };
        match rule.as_str().map(Regex::new) {
            Some(Ok(regex)) => self.expect(
                regex.is_match(text),
                instance,
                &format!("does not match {rule}"),
            ),
            Some(Err(_)) | None => {
                self.report(instance, &format!("has a pattern {rule} that is no regex"));
            }
        }
    }

    fn required(&mut self, rule: &Value, instance: &Instance<'_>) {
        let Some(object) = instance.value.as_object() else {
            return;
        };
        for key in rule
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if !object.contains_key(key) {
                self.report(instance, &format!("lacks the required key {key:?}"));
            }
        }
    }

    fn properties(&mut self, rule: &Value, instance: &Instance<'_>) {
        let (Some(schemas), Some(object)) = (rule.as_object(), instance.value.as_object()) else {
            return;
        };
        for (key, member) in object {
            if let Some(schema) = schemas.get(key) {
                self.check(schema, &instance.child(key, member));
            }
        }
    }

    /// Only `false`, the one form the schema uses: no key outside `properties`.
    fn additional_properties(
        &mut self,
        rule: &Value,
        keywords: &Map<String, Value>,
        instance: &Instance<'_>,
    ) {
        if rule != &Value::Bool(false) {
            self.report(
                instance,
                &format!("has an unsupported additionalProperties {rule}"),
            );
            return;
        }
        let declared = keywords.get("properties").and_then(Value::as_object);
        for key in instance.value.as_object().into_iter().flat_map(Map::keys) {
            if !declared.is_some_and(|properties| properties.contains_key(key)) {
                self.report(
                    instance,
                    &format!("has the key {key:?}, which the schema does not declare"),
                );
            }
        }
    }

    fn items(&mut self, rule: &Value, instance: &Instance<'_>) {
        for (index, item) in instance.value.as_array().into_iter().flatten().enumerate() {
            self.check(rule, &instance.child(&index.to_string(), item));
        }
    }

    fn expect(&mut self, holds: bool, instance: &Instance<'_>, problem: &str) {
        if !holds {
            self.report(instance, problem);
        }
    }

    fn report(&mut self, instance: &Instance<'_>, problem: &str) {
        self.found
            .push(format!("{}: the value {problem}", instance.at));
    }
}

fn is_of_type(rule: &Value, value: &Value) -> bool {
    let names: Vec<&str> = match rule {
        Value::String(name) => vec![name.as_str()],
        Value::Array(names) => names.iter().filter_map(Value::as_str).collect(),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::Object(_) => Vec::new(),
    };
    names.into_iter().any(|name| match name {
        "null" => value.is_null(),
        "boolean" => value.is_boolean(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        "string" => value.is_string(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        _ => false,
    })
}

fn is_listed(rule: &Value, value: &Value) -> bool {
    rule.as_array().is_some_and(|values| values.contains(value))
}

/// The keyword constrains numbers only.
fn is_at_least(rule: &Value, value: &Value) -> bool {
    match (rule.as_f64(), value.as_f64()) {
        (Some(minimum), Some(number)) => number >= minimum,
        _ => true,
    }
}

/// The keyword constrains strings only, and counts characters, not bytes.
fn is_long_enough(rule: &Value, value: &Value) -> bool {
    match (rule.as_u64(), value.as_str()) {
        (Some(minimum), Some(text)) => {
            u64::try_from(text.chars().count()).is_ok_and(|length| length >= minimum)
        }
        _ => true,
    }
}
