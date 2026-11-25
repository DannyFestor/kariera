//! `kariera export --json`: the public career data for the owner's website, as JSON on stdout (issue #36). It prints
//! nothing unless the vault validates and the export gives no confidential Project away.

use std::collections::BTreeSet;
use std::path::Path;

use kariera_documents::export;
use kariera_domain::Career;
use kariera_validate::{disclosures, validate};
use kariera_vault::Vault;
use serde_json::Value;

use crate::clock::current_month;
use crate::reply::Reply;
use crate::validate::counted;
use crate::vault_access::vault_root;

/// The arguments after `export`, which must ask for JSON, the one format there is. The error is what to tell the
/// user about them.
pub(crate) fn parse(arguments: impl IntoIterator<Item = String>) -> Result<(), String> {
    let mut json = false;
    for argument in arguments {
        match argument.as_str() {
            "--json" => json = true,
            _ => return Err(format!("unknown argument {argument:?}")),
        }
    }
    if json {
        Ok(())
    } else {
        Err("export needs --json".to_owned())
    }
}

pub(crate) fn reply() -> Reply {
    match public_career() {
        Ok(json) => Reply::success(json),
        Err(message) => Reply::failure(message),
    }
}

fn public_career() -> Result<String, String> {
    let root = vault_root()?;
    refuse_invalid(&root)?;
    let vault = Vault::load(&root).map_err(|error| error.to_string())?;
    let exported = export(vault.career()).map_err(|error| error.to_string())?;
    let tree = serde_json::to_value(&exported).map_err(|error| error.to_string())?;
    refuse_disclosures(vault.career(), &tree)?;
    // Written from the model rather than the tree, whose objects would sort their keys.
    serde_json::to_string_pretty(&exported).map_err(|error| error.to_string())
}

fn refuse_invalid(root: &Path) -> Result<(), String> {
    let report = validate(root, current_month()?).map_err(|error| error.to_string())?;
    if !report.has_errors() {
        return Ok(());
    }
    let them = if report.errors() == 1 { "it" } else { "them" };
    Err(format!(
        "The vault has {}, so nothing was exported: `kariera validate` lists {them}.",
        counted(report.errors(), "error")
    ))
}

/// Schema §1.9 over every string of the export, note names included: the website keys its own data by them.
fn refuse_disclosures(career: &Career, tree: &Value) -> Result<(), String> {
    let texts: BTreeSet<&str> = strings(tree).into_iter().collect();
    let found = disclosures(career, texts);
    if found.is_empty() {
        return Ok(());
    }
    let lines: Vec<String> = found.iter().map(ToString::to_string).collect();
    Err(format!(
        "The export would give away a confidential Project, so it was not written:\n{}",
        lines.join("\n")
    ))
}

fn strings(value: &Value) -> Vec<&str> {
    match value {
        Value::String(text) => vec![text.as_str()],
        Value::Array(items) => items.iter().flat_map(strings).collect(),
        Value::Object(members) => members.values().flat_map(strings).collect(),
        Value::Null | Value::Bool(_) | Value::Number(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{parse, strings};

    fn parsed(arguments: &[&str]) -> Result<(), String> {
        parse(arguments.iter().map(ToString::to_string))
    }

    #[test]
    fn export_takes_json() {
        assert_eq!(parsed(&["--json"]), Ok(()));
    }

    #[test]
    fn export_without_json_or_with_another_argument_is_refused() {
        assert_eq!(parsed(&[]), Err("export needs --json".to_owned()));
        assert_eq!(
            parsed(&["--json", "--yaml"]),
            Err("unknown argument \"--yaml\"".to_owned())
        );
    }

    #[test]
    fn every_string_value_of_the_tree_is_found_and_no_key() {
        let tree = json!({
            "note": "Acme - Shop",
            "end": null,
            "team_size": 5,
            "skills": ["Go", {"name": "Japanese"}],
        });

        let mut found = strings(&tree);
        found.sort_unstable();

        assert_eq!(found, ["Acme - Shop", "Go", "Japanese"]);
    }
}
