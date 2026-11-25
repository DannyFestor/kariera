//! The lines of the 学歴・職歴 and 免許・資格 tables, each dated by its month (owner's wording on issue #22).

use kariera_domain::{Engagement, EngagementType, Month, Qualification, QualificationCategory};
use serde::Serialize;

use crate::context::{Context, Named};

/// A row of a 年 | 月 | text table, the year in 西暦.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistoryLine {
    pub year: String,
    pub month: String,
    pub text: String,
}

/// Degrees, by the month of each line: 入学 at `start`, 卒業 at `date`.
pub(super) fn education(context: &Context<'_>) -> Vec<HistoryLine> {
    let lines = context
        .qualifications_oldest_first()
        .into_iter()
        .filter(|(_, qualification)| qualification.category == QualificationCategory::Degree)
        .flat_map(|(_, qualification)| degree_lines(context, qualification));
    chronological(lines)
}

fn degree_lines(context: &Context<'_>, qualification: &Qualification) -> Vec<(Month, String)> {
    let school = qualification
        .issuer
        .as_ref()
        .map(|issuer| context.issuer(issuer));
    let degree = context.label(&qualification.name, &qualification.labels);
    let entered = qualification
        .start
        .map(|start| (start, at_school(school.as_deref(), "入学")));
    let graduated = (
        qualification.date,
        at_school(school.as_deref(), &format!("卒業（{degree}）")),
    );
    entered.into_iter().chain([graduated]).collect()
}

fn at_school(school: Option<&str>, event: &str) -> String {
    school.map_or_else(|| event.to_owned(), |school| format!("{school} {event}"))
}

/// Engagements, by the month of each line; an ongoing one has no end line.
pub(super) fn career(context: &Context<'_>) -> Vec<HistoryLine> {
    let mut engagements = context.engagements_newest_first();
    engagements.reverse();
    let lines = engagements
        .into_iter()
        .flat_map(|engagement| engagement_lines(context, engagement));
    chronological(lines)
}

/// Whether the history ends with 現在に至る: only while an Engagement is still running (owner's decision on #34).
pub(super) fn any_ongoing<'career>(
    engagements: impl IntoIterator<Item = &'career Engagement>,
) -> bool {
    engagements
        .into_iter()
        .any(|engagement| engagement.end.is_none())
}

fn engagement_lines(
    context: &Context<'_>,
    (name, engagement): Named<'_, Engagement>,
) -> Vec<(Month, String)> {
    let organization = context.engagement_organization(engagement);
    let role = context.optional_text(name, engagement.role.as_ref());
    let agencies = context.agencies(engagement);
    let started = (
        engagement.start,
        start_text(engagement.engagement_type, &organization, role, &agencies),
    );
    let ended = engagement
        .end
        .map(|end| (end, end_text(engagement.engagement_type, &organization)));
    std::iter::once(started).chain(ended).collect()
}

fn start_text(
    engagement_type: EngagementType,
    organization: &str,
    role: Option<String>,
    agencies: &[String],
) -> String {
    match engagement_type {
        EngagementType::Employment => format!("{organization} 入社{}", brackets([role])),
        EngagementType::Dispatch => format!(
            "{organization} 入社{}",
            brackets([Some("派遣社員".to_owned()), role])
        ),
        EngagementType::Internship => {
            format!("{organization} インターンシップ{}", brackets([role]))
        }
        EngagementType::Freelance => format!(
            "{organization} と業務委託契約{}",
            brackets([Some("フリーランス".to_owned()), via(agencies), role])
        ),
        EngagementType::OwnBusiness => format!("個人事業 {organization} 開業"),
    }
}

fn end_text(engagement_type: EngagementType, organization: &str) -> String {
    let event = match engagement_type {
        EngagementType::Employment | EngagementType::Dispatch => "退社",
        EngagementType::Internship => "インターンシップ修了",
        EngagementType::Freelance => "契約満了",
        EngagementType::OwnBusiness => "廃業",
    };
    format!("{organization} {event}")
}

fn via(agencies: &[String]) -> Option<String> {
    (!agencies.is_empty()).then(|| format!("{}経由", agencies.join("・")))
}

/// The parts that are set, in Japanese brackets; nothing when none is.
fn brackets<const N: usize>(parts: [Option<String>; N]) -> String {
    let parts: Vec<String> = parts.into_iter().flatten().collect();
    if parts.is_empty() {
        String::new()
    } else {
        format!("（{}）", parts.join("、"))
    }
}

/// Every Qualification but degrees, by `date`.
pub(super) fn licences(context: &Context<'_>) -> Vec<HistoryLine> {
    let lines = context
        .qualifications_oldest_first()
        .into_iter()
        .filter(|(_, qualification)| qualification.category != QualificationCategory::Degree)
        .map(|(_, qualification)| (qualification.date, licence_text(context, qualification)));
    chronological(lines)
}

fn licence_text(context: &Context<'_>, qualification: &Qualification) -> String {
    let category = qualification.category;
    context
        .qualification_entry(qualification, |label, level, score| {
            licence_wording(category, &label, level, score)
        })
        .title
}

fn licence_wording(
    category: QualificationCategory,
    label: &str,
    level: Option<String>,
    score: Option<String>,
) -> String {
    match (category, score) {
        (QualificationCategory::TestResult, Some(score)) => format!("{label} {score} 取得"),
        (QualificationCategory::TestResult, None) => {
            let level = level.map_or_else(String::new, |level| format!(" {level}"));
            format!("{label}{level} 合格")
        }
        (
            QualificationCategory::Degree
            | QualificationCategory::Certificate
            | QualificationCategory::Licence,
            _,
        ) => format!("{label} 取得"),
    }
}

/// Ordered by month; lines of one month keep the order they came in.
fn chronological(lines: impl Iterator<Item = (Month, String)>) -> Vec<HistoryLine> {
    let mut lines: Vec<(Month, String)> = lines.collect();
    lines.sort_by_key(|(month, _)| *month);
    lines
        .into_iter()
        .map(|(month, text)| HistoryLine {
            year: month.year().to_string(),
            month: month.number().to_string(),
            text,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use kariera_domain::{EngagementType, QualificationCategory};

    use super::{any_ongoing, at_school, end_text, licence_wording, start_text};
    use crate::test_career::engagement;

    #[test]
    fn an_engagement_without_an_end_is_ongoing() {
        let ended = engagement("2019-04", Some("2021-03"));
        let running = engagement("2021-04", None);

        assert!(any_ongoing([&ended, &running]));
        assert!(!any_ongoing([&ended]));
        assert!(!any_ongoing([]));
    }

    #[test]
    fn each_engagement_type_starts_with_its_own_wording() {
        let agencies = ["Fabrikam".to_owned()];
        let cases = [
            (EngagementType::Employment, "Acme 入社（開発者）"),
            (EngagementType::Dispatch, "Acme 入社（派遣社員、開発者）"),
            (
                EngagementType::Internship,
                "Acme インターンシップ（開発者）",
            ),
            (
                EngagementType::Freelance,
                "Acme と業務委託契約（フリーランス、Fabrikam経由、開発者）",
            ),
            (EngagementType::OwnBusiness, "個人事業 Acme 開業"),
        ];
        for (engagement_type, expected) in cases {
            assert_eq!(
                start_text(
                    engagement_type,
                    "Acme",
                    Some("開発者".to_owned()),
                    &agencies
                ),
                expected
            );
        }
    }

    #[test]
    fn a_missing_role_and_no_agency_leave_out_their_parts() {
        assert_eq!(
            start_text(EngagementType::Employment, "Acme", None, &[]),
            "Acme 入社"
        );
        assert_eq!(
            start_text(EngagementType::Internship, "Acme", None, &[]),
            "Acme インターンシップ"
        );
        assert_eq!(
            start_text(EngagementType::Freelance, "Acme", None, &[]),
            "Acme と業務委託契約（フリーランス）"
        );
        assert_eq!(
            start_text(
                EngagementType::Freelance,
                "Acme",
                None,
                &["Fabrikam".to_owned(), "Contoso".to_owned()]
            ),
            "Acme と業務委託契約（フリーランス、Fabrikam・Contoso経由）"
        );
    }

    #[test]
    fn each_engagement_type_ends_with_its_own_wording() {
        let cases = [
            (EngagementType::Employment, "Acme 退社"),
            (EngagementType::Dispatch, "Acme 退社"),
            (EngagementType::Internship, "Acme インターンシップ修了"),
            (EngagementType::Freelance, "Acme 契約満了"),
            (EngagementType::OwnBusiness, "Acme 廃業"),
        ];
        for (engagement_type, expected) in cases {
            assert_eq!(end_text(engagement_type, "Acme"), expected);
        }
    }

    #[test]
    fn a_degree_without_a_school_names_only_the_event() {
        assert_eq!(
            at_school(Some("エグザンプル大学"), "入学"),
            "エグザンプル大学 入学"
        );
        assert_eq!(at_school(None, "入学"), "入学");
    }

    #[test]
    fn a_licence_or_certificate_is_obtained() {
        for category in [
            QualificationCategory::Licence,
            QualificationCategory::Certificate,
        ] {
            assert_eq!(
                licence_wording(category, "普通自動車免許", None, None),
                "普通自動車免許 取得"
            );
        }
    }

    #[test]
    fn a_test_result_is_obtained_with_its_score_or_passed_at_its_level() {
        let label = "日本語試験";

        assert_eq!(
            licence_wording(
                QualificationCategory::TestResult,
                label,
                Some("B2".to_owned()),
                Some("185/200".to_owned())
            ),
            "日本語試験 185/200 取得"
        );
        assert_eq!(
            licence_wording(
                QualificationCategory::TestResult,
                label,
                Some("N1".to_owned()),
                None
            ),
            "日本語試験 N1 合格"
        );
        assert_eq!(
            licence_wording(QualificationCategory::TestResult, label, None, None),
            "日本語試験 合格"
        );
    }
}
