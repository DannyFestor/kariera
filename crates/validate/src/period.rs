use kariera_domain::{Contract, Date, Engagement, Month, Project};

/// The months from `start` to `end`, both included; open-ended without an `end` (schema §1.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Period {
    pub(crate) start: Month,
    pub(crate) end: Option<Month>,
}

impl Period {
    pub(crate) const fn of_engagement(engagement: &Engagement) -> Self {
        Self {
            start: engagement.start,
            end: engagement.end,
        }
    }

    pub(crate) const fn of_project(project: &Project) -> Self {
        Self {
            start: project.start,
            end: project.end,
        }
    }

    pub(crate) fn of_contract(contract: &Contract) -> Self {
        Self {
            start: contract.start.month(),
            end: contract.end.map(Date::month),
        }
    }

    pub(crate) fn contains(self, month: Month) -> bool {
        self.explain_why_outside(month).is_none()
    }

    /// Completes "… the Engagement": why `month` is not in the period.
    pub(crate) fn explain_why_outside(self, month: Month) -> Option<String> {
        if month < self.start {
            Some(format!("is before the start ({}) of", self.start))
        } else {
            self.end
                .filter(|end| month > *end)
                .map(|end| format!("is after the end ({end}) of"))
        }
    }
}

#[cfg(test)]
mod tests {
    use kariera_domain::Month;

    use super::Period;

    fn month(text: &str) -> Month {
        text.parse().unwrap()
    }

    fn period(start: &str, end: Option<&str>) -> Period {
        Period {
            start: month(start),
            end: end.map(month),
        }
    }

    #[test]
    fn a_period_includes_both_its_months() {
        let period = period("2024-05", Some("2024-07"));

        assert!(period.contains(month("2024-05")));
        assert!(period.contains(month("2024-07")));
        assert!(!period.contains(month("2024-04")));
        assert!(!period.contains(month("2024-08")));
    }

    #[test]
    fn a_period_without_an_end_has_no_last_month() {
        assert!(period("2024-05", None).contains(month("2099-12")));
    }

    #[test]
    fn a_month_outside_is_explained_by_the_bound_it_crosses() {
        let period = period("2024-05", Some("2024-07"));

        assert_eq!(
            period.explain_why_outside(month("2024-04")).as_deref(),
            Some("is before the start (2024-05) of")
        );
        assert_eq!(
            period.explain_why_outside(month("2024-08")).as_deref(),
            Some("is after the end (2024-07) of")
        );
        assert_eq!(period.explain_why_outside(month("2024-06")), None);
    }
}
