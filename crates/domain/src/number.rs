use std::fmt;
use std::iter::Sum;
use std::str::FromStr;

use crate::ValueError;

/// The schema's Number (§1.2), which only hours use. Kept in hundredths, so sums are exact.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hours {
    hundredths: u32,
}

/// The schema's Int (§1.2) where a row bounds it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WholeNumber<const MIN: u32, const MAX: u32>(u32);

/// Total team including the owner (schema §2.4).
pub type TeamSize = WholeNumber<1, { u32::MAX }>;
pub type DaysPerWeek = WholeNumber<1, 7>;
/// JPY excluding consumption tax, in the Contract's `rate_unit` (schema §1.10).
pub type Yen = WholeNumber<0, { u32::MAX }>;

impl<const MIN: u32, const MAX: u32> WholeNumber<MIN, MAX> {
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    const fn out_of_range() -> ValueError {
        ValueError::OutOfRange { min: MIN, max: MAX }
    }
}

impl<const MIN: u32, const MAX: u32> FromStr for WholeNumber<MIN, MAX> {
    type Err = ValueError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let value = digits(text)
            .and_then(|digits| digits.parse::<u32>().ok())
            .ok_or_else(Self::out_of_range)?;
        (MIN..=MAX)
            .contains(&value)
            .then_some(Self(value))
            .ok_or_else(Self::out_of_range)
    }
}

impl FromStr for Hours {
    type Err = ValueError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (whole, fraction) = text.split_once('.').unwrap_or((text, "0"));
        let whole = digits(whole).and_then(|whole| whole.parse::<u32>().ok());
        let fraction = digits(fraction).and_then(hundredths_of_fraction);
        whole
            .zip(fraction)
            .and_then(|(whole, fraction)| whole.checked_mul(100)?.checked_add(fraction))
            .map(|hundredths| Self { hundredths })
            .ok_or(ValueError::NotHours)
    }
}

fn digits(text: &str) -> Option<&str> {
    let is_digits = !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit());
    is_digits.then_some(text)
}

fn hundredths_of_fraction(fraction: &str) -> Option<u32> {
    let value = fraction.parse::<u32>().ok()?;
    match fraction.len() {
        1 => Some(value * 10),
        2 => Some(value),
        _ => None,
    }
}

impl Sum for Hours {
    fn sum<I: Iterator<Item = Self>>(hours: I) -> Self {
        let hundredths = hours.fold(0, |total: u32, hours| {
            total.saturating_add(hours.hundredths)
        });
        Self { hundredths }
    }
}

impl fmt::Display for Hours {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let whole = self.hundredths / 100;
        let fraction = self.hundredths % 100;
        if fraction == 0 {
            write!(formatter, "{whole}")
        } else if fraction.is_multiple_of(10) {
            write!(formatter, "{whole}.{}", fraction / 10)
        } else {
            write!(formatter, "{whole}.{fraction:02}")
        }
    }
}

// The written form, so a snapshot of a whole note stays readable.
impl fmt::Debug for Hours {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl<const MIN: u32, const MAX: u32> fmt::Debug for WholeNumber<MIN, MAX> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::{DaysPerWeek, Hours, TeamSize, Yen};
    use crate::ValueError;

    #[test]
    fn hours_read_and_write_with_up_to_two_decimals() {
        for text in ["0", "132", "152.5", "30.25", "7.05", "1000000"] {
            assert_eq!(text.parse::<Hours>().unwrap().to_string(), text);
        }
    }

    #[test]
    fn trailing_zeros_of_hours_are_dropped() {
        assert_eq!(format!("{:?}", "30.50".parse::<Hours>().unwrap()), "30.5");
        assert_eq!("12.0".parse::<Hours>().unwrap().to_string(), "12");
    }

    #[test]
    fn hours_add_up_exactly() {
        let hours = ["30.25", "100", "0.1"].map(|text| text.parse::<Hours>().unwrap());

        assert_eq!(hours.into_iter().sum::<Hours>().to_string(), "130.35");
        assert_eq!(std::iter::empty::<Hours>().sum::<Hours>().to_string(), "0");
    }

    #[test]
    fn hours_that_overflow_stay_at_the_largest_value() {
        let largest = "42949672.95".parse::<Hours>().unwrap();

        assert_eq!([largest, largest].into_iter().sum::<Hours>(), largest);
    }

    #[test]
    fn hours_compare_by_value() {
        assert!("30.25".parse::<Hours>().unwrap() < "30.3".parse::<Hours>().unwrap());
        assert_eq!("30.5".parse::<Hours>(), "30.50".parse::<Hours>());
    }

    #[test]
    fn hours_in_another_shape_are_refused() {
        for text in [
            "", "-1", "+1", "1.234", "1.", ".5", "1e2", "1,5", "a", "1.-5", "42949673",
        ] {
            assert_eq!(text.parse::<Hours>(), Err(ValueError::NotHours), "{text}");
        }
    }

    #[test]
    fn a_whole_number_inside_its_bounds_is_read() {
        assert_eq!("7".parse::<DaysPerWeek>().map(DaysPerWeek::get), Ok(7));
        assert_eq!("1".parse::<TeamSize>().map(TeamSize::get), Ok(1));
        assert_eq!(format!("{:?}", "0".parse::<Yen>().unwrap()), "0");
    }

    #[test]
    fn a_whole_number_outside_its_bounds_is_refused() {
        let days = ValueError::OutOfRange { min: 1, max: 7 };

        assert_eq!("0".parse::<DaysPerWeek>(), Err(days.clone()));
        assert_eq!("8".parse::<DaysPerWeek>(), Err(days));
    }

    #[test]
    fn anything_but_digits_is_not_a_whole_number() {
        for text in ["", "-1", "+1", "1.0", "1e3", "99999999999"] {
            assert!(text.parse::<Yen>().is_err(), "{text}");
        }
    }
}
