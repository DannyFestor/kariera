use std::fmt;
use std::str::FromStr;

use crate::ValueError;

/// Written `YYYY-MM` (schema §1.3).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Month {
    year: u16,
    month: u8,
}

/// Written `YYYY-MM-DD` (schema §1.3).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    month: Month,
    day: u8,
}

impl Month {
    fn new(year: u16, month: u8) -> Option<Self> {
        (1..=12).contains(&month).then_some(Self { year, month })
    }

    const fn is_in_leap_year(self) -> bool {
        self.year.is_multiple_of(4)
            && (!self.year.is_multiple_of(100) || self.year.is_multiple_of(400))
    }

    const fn days(self) -> u8 {
        match self.month {
            2 if self.is_in_leap_year() => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        }
    }
}

impl Date {
    #[must_use]
    pub const fn month(self) -> Month {
        self.month
    }

    fn new(month: Month, day: u8) -> Option<Self> {
        (1..=month.days())
            .contains(&day)
            .then_some(Self { month, day })
    }
}

impl FromStr for Month {
    type Err = ValueError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (year, month) = text.split_once('-').ok_or(ValueError::NotAMonth)?;
        let year = fixed_width_number(year, 4).ok_or(ValueError::NotAMonth)?;
        let month = fixed_width_number(month, 2).ok_or(ValueError::NotAMonth)?;
        Self::new(year, month).ok_or(ValueError::NotAMonth)
    }
}

impl FromStr for Date {
    type Err = ValueError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (month, day) = text.rsplit_once('-').ok_or(ValueError::NotADate)?;
        let month = month.parse::<Month>().ok();
        let day = fixed_width_number(day, 2);
        month
            .zip(day)
            .and_then(|(month, day)| Self::new(month, day))
            .ok_or(ValueError::NotADate)
    }
}

fn fixed_width_number<T: FromStr>(digits: &str, width: usize) -> Option<T> {
    let is_fixed_width = digits.len() == width && digits.bytes().all(|byte| byte.is_ascii_digit());
    is_fixed_width.then(|| digits.parse().ok()).flatten()
}

impl fmt::Display for Month {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:04}-{:02}", self.year, self.month)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}-{:02}", self.month, self.day)
    }
}

// The written form, so a snapshot of a whole note stays readable.
impl fmt::Debug for Month {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Debug for Date {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::{Date, Month};
    use crate::ValueError;

    fn month(text: &str) -> Month {
        text.parse().unwrap()
    }

    #[test]
    fn a_month_reads_and_writes_as_yyyy_mm() {
        assert_eq!(month("2024-05").to_string(), "2024-05");
        assert_eq!(format!("{:?}", month("0999-12")), "0999-12");
    }

    #[test]
    fn a_month_outside_1_to_12_is_refused() {
        assert_eq!("2024-00".parse::<Month>(), Err(ValueError::NotAMonth));
        assert_eq!("2024-13".parse::<Month>(), Err(ValueError::NotAMonth));
    }

    #[test]
    fn a_month_in_another_shape_is_refused() {
        for text in [
            "2024-5",
            "24-05",
            "2024/05",
            "2024-05-01",
            "+024-05",
            "2024-+5",
            "",
        ] {
            assert_eq!(text.parse::<Month>(), Err(ValueError::NotAMonth), "{text}");
        }
    }

    #[test]
    fn months_order_by_year_then_month() {
        assert!(month("2023-12") < month("2024-01"));
        assert!(month("2024-01") < month("2024-02"));
    }

    #[test]
    fn a_date_reads_and_writes_as_yyyy_mm_dd() {
        let date = "2026-03-04".parse::<Date>().unwrap();

        assert_eq!(date.to_string(), "2026-03-04");
        assert_eq!(format!("{date:?}"), "2026-03-04");
        assert_eq!(date.month(), month("2026-03"));
    }

    #[test]
    fn a_date_knows_the_length_of_its_month() {
        for text in [
            "2025-01-31",
            "2025-04-30",
            "2025-02-28",
            "2024-02-29",
            "2000-02-29",
        ] {
            assert!(text.parse::<Date>().is_ok(), "{text}");
        }
        for text in [
            "2025-01-32",
            "2025-04-31",
            "2025-02-29",
            "1900-02-29",
            "2025-06-00",
        ] {
            assert_eq!(text.parse::<Date>(), Err(ValueError::NotADate), "{text}");
        }
    }

    #[test]
    fn a_date_in_another_shape_is_refused() {
        for text in [
            "2026-3-04",
            "2026-03-4",
            "2026-03",
            "20260304",
            "2026-13-01",
        ] {
            assert_eq!(text.parse::<Date>(), Err(ValueError::NotADate), "{text}");
        }
    }

    #[test]
    fn dates_order_by_day_within_a_month() {
        let earlier = "2026-03-04".parse::<Date>().unwrap();
        let later = "2026-03-05".parse::<Date>().unwrap();

        assert!(earlier < later);
    }
}
