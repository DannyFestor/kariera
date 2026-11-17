use std::ops::RangeInclusive;

use kariera_domain::Month;

/// The months from `first` to `last`, both counted; empty when `last` is before `first`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MonthSpan {
    pub(crate) first: Month,
    pub(crate) last: Month,
}

impl MonthSpan {
    /// A period that is still running lasts until `today`.
    pub(crate) fn of(start: Month, end: Option<Month>, today: Month) -> Self {
        Self {
            first: start,
            last: end.unwrap_or(today),
        }
    }

    pub(crate) fn ordinals(self) -> RangeInclusive<u32> {
        ordinal(self.first)..=ordinal(self.last)
    }

    pub(crate) fn len(self) -> u32 {
        (ordinal(self.last) + 1).saturating_sub(ordinal(self.first))
    }
}

/// Months counted from January of year 0, so the distance between two months is a subtraction.
pub(crate) fn ordinal(month: Month) -> u32 {
    u32::from(month.year()) * 12 + u32::from(month.number()) - 1
}

/// Whether `month` lies within the `years` that end with `today`, `today` included.
pub(crate) fn is_within_years(month: Month, years: u32, today: Month) -> bool {
    ordinal(month) + years * 12 > ordinal(today)
}

#[cfg(test)]
mod tests {
    use kariera_domain::Month;

    use super::{MonthSpan, is_within_years};

    fn month(text: &str) -> Month {
        text.parse().unwrap()
    }

    #[test]
    fn a_span_counts_its_first_and_last_month() {
        let span = MonthSpan::of(month("2024-11"), Some(month("2025-02")), month("2026-11"));

        assert_eq!(span.len(), 4);
        assert_eq!(span.ordinals().count(), 4);
    }

    #[test]
    fn a_running_span_lasts_until_today() {
        let span = MonthSpan::of(month("2026-01"), None, month("2026-11"));

        assert_eq!(span.len(), 11);
    }

    #[test]
    fn a_span_that_starts_after_it_ends_is_empty() {
        let span = MonthSpan::of(month("2027-01"), None, month("2026-11"));

        assert_eq!(span.len(), 0);
        assert_eq!(span.ordinals().count(), 0);
    }

    #[test]
    fn three_years_up_to_november_2026_start_in_december_2023() {
        let today = month("2026-11");

        assert!(is_within_years(month("2023-12"), 3, today));
        assert!(!is_within_years(month("2023-11"), 3, today));
        assert!(is_within_years(today, 3, today));
    }
}
