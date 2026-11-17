//! Schema §2.6: a Skill is not last used after the current month.

use kariera_domain::Skill;

use crate::context::Context;
use crate::field_path::FieldPath;
use crate::finding::{At, Finding, Rule};

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    let current_month = context.current_month();
    context
        .notes::<Skill>()
        .filter_map(|skill| {
            let last_used = skill
                .note
                .last_used
                .filter(|month| *month > current_month)?;
            Some(Finding::new(
                skill.file,
                Rule::Period,
                At::Field(FieldPath::of("last_used")),
                format!("{last_used} is after the current month ({current_month})"),
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::test_vault::TestVault;

    #[test]
    fn a_skill_last_used_in_the_current_month_is_fine() {
        let vault = TestVault::new("last_used_current");
        vault.replace(
            "Skills/Docker.md",
            "last_used: 2025-08",
            "last_used: 2026-11",
        );

        assert_eq!(vault.lines(), Vec::<String>::new());
    }

    #[test]
    fn a_skill_last_used_after_the_current_month_is_reported() {
        let vault = TestVault::new("last_used_future");
        vault.replace(
            "Skills/Docker.md",
            "last_used: 2025-08",
            "last_used: 2026-12",
        );

        assert_eq!(
            vault.lines(),
            [
                "Skills/Docker.md:5: error [period] field `last_used` 2026-12 is after the current month (2026-11)"
            ]
        );
    }
}
