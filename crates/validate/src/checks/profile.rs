//! Schema §2.8 and §3: exactly one Profile, whose days per week do not run backwards.

use std::path::Path;

use kariera_domain::Profile;

use crate::context::{Context, Found};
use crate::field_path::FieldPath;
use crate::finding::{At, Finding, Rule};

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    let mut findings = Vec::new();
    if context.is_complete() {
        findings.extend(exactly_one(context));
    }
    findings.extend(
        context
            .notes::<Profile>()
            .filter_map(|profile| days_per_week_in_order(&profile)),
    );
    findings
}

fn exactly_one(context: &Context<'_>) -> Option<Finding> {
    let profiles = context.career().profiles.len();
    (profiles != 1).then(|| {
        Finding::at_path(
            Path::new("Profile.md"),
            Rule::Profile,
            format!("must exist exactly once in the vault root, but the vault has {profiles} Profile notes"),
        )
    })
}

fn days_per_week_in_order(profile: &Found<'_, Profile>) -> Option<Finding> {
    let min = profile.note.days_per_week_min?;
    let max = profile.note.days_per_week_max?;
    (max < min).then(|| {
        Finding::new(
            profile.file,
            Rule::Profile,
            At::Field(FieldPath::of("days_per_week_max")),
            format!("must not be below `days_per_week_min` ({min:?})"),
        )
    })
}

#[cfg(test)]
mod tests {
    use crate::test_vault::{TestVault, assert_reports};

    #[test]
    fn a_vault_without_a_profile_is_reported() {
        let vault = TestVault::new("profile_missing");
        vault.remove("Profile.md");

        assert_eq!(
            vault.lines(),
            [
                "Profile.md: error [profile] the note must exist exactly once in the vault root, but the vault has 0 Profile notes"
            ]
        );
    }

    #[test]
    fn a_vault_with_two_profiles_is_reported() {
        let vault = TestVault::new("profile_two");
        vault.write(
            "Skills/Second.md",
            "---\ntype: profile\nname: Second\nheadline:\n  en: Developer\nbusiness: \"[[Example Works]]\"\nreviewed: false\n---\n\n## Summary\n\n## Strengths\n\n## Self-PR\n\n## 日本語\n\n### Summary\n\n### Strengths\n\n### Self-PR\n\n## Deutsch\n\n### Summary\n\n### Strengths\n\n### Self-PR\n",
        );

        assert_reports(
            &vault,
            "Profile.md: error [profile] the note must exist exactly once in the vault root, but the vault has 2 Profile notes",
        );
    }

    #[test]
    fn the_maximum_days_per_week_is_not_below_the_minimum() {
        let vault = TestVault::new("profile_days");
        vault.replace("Profile.md", "days_per_week_max: 5", "days_per_week_max: 2");

        assert_eq!(
            vault.lines(),
            [
                "Profile.md:11: error [profile] field `days_per_week_max` must not be below `days_per_week_min` (3)"
            ]
        );
    }

    #[test]
    fn equal_days_per_week_are_fine() {
        let vault = TestVault::new("profile_days_equal");
        vault.replace("Profile.md", "days_per_week_max: 5", "days_per_week_max: 3");

        assert_eq!(vault.lines(), Vec::<String>::new());
    }
}
