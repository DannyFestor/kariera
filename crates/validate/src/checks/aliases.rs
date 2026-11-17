//! Schema §2.4 and §3: no alias belongs to two Projects, ignoring case.

use std::collections::BTreeMap;

use kariera_domain::{NoteName, Project};
use kariera_vault::nfc;

use crate::context::Context;
use crate::field_path::FieldPath;
use crate::finding::{At, Finding, Rule};

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    let mut owner_of: BTreeMap<String, &NoteName> = BTreeMap::new();
    let mut findings = Vec::new();
    for project in context.notes::<Project>() {
        for (index, alias) in project.note.aliases.iter().enumerate() {
            let key = nfc(alias.as_str()).to_lowercase();
            match owner_of.get(&key) {
                Some(owner) if *owner != project.name => findings.push(Finding::new(
                    project.file,
                    Rule::Alias,
                    At::Field(FieldPath::of("aliases").item(index)),
                    format!("is also an alias of the Project [[{owner}]], ignoring case"),
                )),
                Some(_) => {}
                None => {
                    owner_of.insert(key, project.name);
                }
            }
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use crate::test_vault::TestVault;

    #[test]
    fn an_alias_of_two_projects_is_reported_ignoring_case() {
        let vault = TestVault::new("aliases_shared");
        vault.replace(
            "Projects/Globex - Admin.md",
            "confidential: false\n",
            "confidential: false\naliases:\n  - shop\n",
        );

        assert_eq!(
            vault.lines(),
            [
                "Projects/Globex - Admin.md:8: error [alias] field `aliases[0]` is also an alias of the Project [[Acme - Shop]], ignoring case"
            ]
        );
    }

    #[test]
    fn a_project_may_repeat_its_own_alias_in_another_case() {
        let vault = TestVault::new("aliases_own");
        vault.replace(
            "Projects/Acme - Shop.md",
            "  - TOYS-EC\n",
            "  - TOYS-EC\n  - shop\n",
        );

        assert_eq!(vault.lines(), Vec::<String>::new());
    }
}
