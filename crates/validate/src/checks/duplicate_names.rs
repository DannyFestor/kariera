//! Schema §1.1: names are unique across the vault, ignoring case. The vault reader refuses two notes of exactly
//! one name; this finds the ones that differ in case only.

use std::collections::BTreeMap;

use kariera_vault::{NoteFile, nfc};

use crate::context::Context;
use crate::finding::{At, Finding, Rule};

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    let mut first_by_name: BTreeMap<String, &NoteFile> = BTreeMap::new();
    let mut findings = Vec::new();
    for file in context.files() {
        let key = nfc(file.name.as_str()).to_lowercase();
        match first_by_name.get(&key) {
            Some(first) => findings.push(Finding::new(
                file,
                Rule::DuplicateName,
                At::Note,
                format!(
                    "has a name that differs from the name of {} in case only",
                    first.path.display()
                ),
            )),
            None => {
                first_by_name.insert(key, file);
            }
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use crate::test_vault::TestVault;

    #[test]
    fn two_names_that_differ_in_case_only_are_reported_on_the_later_note() {
        let vault = TestVault::new("duplicate_case");
        let go = "---\ntype: skill\nname: GO\ncategory: language\nreviewed: false\n---\n";
        vault.write("Skills/More/GO.md", go);

        assert_eq!(
            vault.lines(),
            [
                "Skills/More/GO.md: error [folder] the note is in `Skills/More/`, but a skill note belongs in `Skills/`",
                "Skills/More/GO.md: error [duplicate-name] the note has a name that differs from the name of Skills/Go.md in case only",
            ]
        );
    }
}
