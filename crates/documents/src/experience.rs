//! Years of experience per Skill and which Skills count as recent (owner's rule on issue #7).

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

use kariera_domain::{Career, Month, NoteName, Skill, SkillCategory};

use crate::months::{MonthSpan, is_within_years};

/// What the Projects of a career say about each Skill they link.
#[derive(Debug)]
pub(crate) struct Experience<'career> {
    career: &'career Career,
    today: Month,
    by_skill: BTreeMap<&'career NoteName, ProjectUse>,
}

#[derive(Debug, Default)]
struct ProjectUse {
    months: BTreeSet<u32>,
    last_month: Option<Month>,
}

/// A Skill in a skill table.
#[derive(Debug, Clone, Copy)]
pub(crate) struct UsedSkill<'career> {
    pub(crate) skill: &'career Skill,
    /// The months of its Projects, a month shared by two Projects counted once (schema §4).
    pub(crate) months: u32,
    /// The later of its last Project month and its `last_used`.
    pub(crate) last_use: Month,
}

impl<'career> Experience<'career> {
    pub(crate) fn of(career: &'career Career, today: Month) -> Self {
        let mut by_skill: BTreeMap<&NoteName, ProjectUse> = BTreeMap::new();
        for project in career.projects.values() {
            let span = MonthSpan::of(project.start, project.end, today);
            for link in &project.skills {
                let entry = by_skill.entry(link.target()).or_default();
                entry.months.extend(span.ordinals());
                entry.last_month = entry.last_month.max(Some(span.last));
            }
        }
        Self {
            career,
            today,
            by_skill,
        }
    }

    /// The Skills a Project links whose last use falls within `years` of today, human languages left out, by
    /// category in schema order and then most months first.
    pub(crate) fn recent(&self, years: u32) -> Vec<UsedSkill<'career>> {
        let mut used: Vec<UsedSkill<'career>> = self
            .career
            .skills
            .iter()
            .filter(|(_, skill)| skill.category != SkillCategory::HumanLanguage)
            .filter_map(|(name, skill)| self.used(name, skill))
            .filter(|used| is_within_years(used.last_use, years, self.today))
            .collect();
        used.sort_by_key(|used| (used.skill.category, Reverse(used.months)));
        used
    }

    fn used(&self, name: &'career NoteName, skill: &'career Skill) -> Option<UsedSkill<'career>> {
        let project_use = self.by_skill.get(name)?;
        let last_month = project_use.last_month?;
        Some(UsedSkill {
            skill,
            months: count(&project_use.months),
            last_use: last_month.max(skill.last_used.unwrap_or(last_month)),
        })
    }
}

fn count(months: &BTreeSet<u32>) -> u32 {
    u32::try_from(months.len()).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use kariera_domain::SkillCategory;

    use super::Experience;
    use crate::test_career::{career, month, project, skill};

    #[test]
    fn overlapping_projects_count_their_shared_months_once() {
        let career = career(
            vec![
                project("2024-01", Some("2024-06"), &["Go"]),
                project("2024-04", Some("2024-09"), &["Go"]),
            ],
            vec![skill("Go", SkillCategory::Language, None)],
        );

        let experience = Experience::of(&career, month("2026-11"));

        assert_eq!(
            experience.recent(5).first().map(|used| used.months),
            Some(9)
        );
    }

    #[test]
    fn a_running_project_counts_until_today() {
        let career = career(
            vec![project("2026-01", None, &["Go"])],
            vec![skill("Go", SkillCategory::Language, None)],
        );

        let used = Experience::of(&career, month("2026-11")).recent(3);

        assert_eq!(used.first().map(|used| used.months), Some(11));
        assert_eq!(
            used.first().map(|used| used.last_use),
            Some(month("2026-11"))
        );
    }

    #[test]
    fn a_skill_last_used_in_a_project_before_the_window_is_left_out() {
        let career = career(
            vec![project("2020-01", Some("2023-11"), &["Go"])],
            vec![skill("Go", SkillCategory::Language, None)],
        );

        assert!(
            Experience::of(&career, month("2026-11"))
                .recent(3)
                .is_empty()
        );
    }

    #[test]
    fn last_used_brings_a_skill_into_the_window_without_adding_months() {
        let career = career(
            vec![project("2020-01", Some("2020-12"), &["Go"])],
            vec![skill("Go", SkillCategory::Language, Some("2025-08"))],
        );

        let used = Experience::of(&career, month("2026-11")).recent(3);

        assert_eq!(used.first().map(|used| used.months), Some(12));
        assert_eq!(
            used.first().map(|used| used.last_use),
            Some(month("2025-08"))
        );
    }

    #[test]
    fn a_skill_no_project_uses_is_left_out_even_when_last_used_is_recent() {
        let career = career(
            vec![project("2025-01", None, &["Go"])],
            vec![
                skill("Go", SkillCategory::Language, None),
                skill("Docker", SkillCategory::Tool, Some("2026-10")),
            ],
        );

        let names: Vec<&str> = Experience::of(&career, month("2026-11"))
            .recent(3)
            .iter()
            .map(|used| used.skill.name.as_str())
            .collect();

        assert_eq!(names, ["Go"]);
    }

    #[test]
    fn human_languages_are_not_in_skill_tables() {
        let career = career(
            vec![project("2025-01", None, &["Japanese", "Go"])],
            vec![
                skill("Japanese", SkillCategory::HumanLanguage, None),
                skill("Go", SkillCategory::Language, None),
            ],
        );

        let names: Vec<&str> = Experience::of(&career, month("2026-11"))
            .recent(3)
            .iter()
            .map(|used| used.skill.name.as_str())
            .collect();

        assert_eq!(names, ["Go"]);
    }

    #[test]
    fn skills_are_ordered_by_category_then_most_months_first() {
        let career = career(
            vec![
                project("2025-01", None, &["Docker", "Go", "Rust"]),
                project("2020-01", None, &["Rust"]),
            ],
            vec![
                skill("Docker", SkillCategory::Tool, None),
                skill("Go", SkillCategory::Language, None),
                skill("Rust", SkillCategory::Language, None),
            ],
        );

        let names: Vec<&str> = Experience::of(&career, month("2026-11"))
            .recent(3)
            .iter()
            .map(|used| used.skill.name.as_str())
            .collect();

        assert_eq!(names, ["Rust", "Go", "Docker"]);
    }
}
