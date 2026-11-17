//! How each language writes months, dates, periods and lengths of time (the conventions of the layout prototypes).

use kariera_domain::{Date, Language, Month};

const ENGLISH_MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Locale(pub(crate) Language);

impl Locale {
    pub(crate) fn month(self, month: Month) -> String {
        let (year, number) = (month.year(), month.number());
        match self.0 {
            Language::English => format!("{} {year}", english_month(number)),
            Language::Japanese => format!("{year}年{number}月"),
            Language::German => format!("{number:02}/{year}"),
        }
    }

    pub(crate) fn date(self, date: Date) -> String {
        let (year, number, day) = (date.month().year(), date.month().number(), date.day());
        match self.0 {
            Language::English => format!("{day} {} {year}", english_month(number)),
            Language::Japanese => format!("{year}年{number}月{day}日"),
            Language::German => format!("{day:02}.{number:02}.{year}"),
        }
    }

    /// An `end` of `None` is a period still running.
    pub(crate) fn period(self, start: Month, end: Option<Month>) -> String {
        let first = self.month(start);
        match (self.0, end) {
            (_, Some(end)) if end == start => first,
            (Language::Japanese, Some(end)) => format!("{first}～{}", self.month(end)),
            (Language::English | Language::German, Some(end)) => {
                format!("{first} – {}", self.month(end))
            }
            (Language::Japanese, None) => format!("{first}～現在"),
            (Language::English, None) => format!("since {first}"),
            (Language::German, None) => format!("seit {first}"),
        }
    }

    /// The length of `months`: years with one decimal in English and German, years and months in Japanese.
    pub(crate) fn experience(self, months: u32) -> String {
        let tenths_of_years = (months * 10 + 6) / 12;
        let (whole, tenth) = (tenths_of_years / 10, tenths_of_years % 10);
        match self.0 {
            Language::English => format!("{whole}.{tenth} y"),
            Language::German => format!("{whole},{tenth} J."),
            Language::Japanese => japanese_duration(months),
        }
    }

    pub(crate) fn range(self, from: u32, to: u32) -> String {
        if from == to {
            return from.to_string();
        }
        match self.0 {
            Language::Japanese => format!("{from}～{to}"),
            Language::English | Language::German => format!("{from}–{to}"),
        }
    }

    /// The paragraphs of a Paragraphs section (schema §1.6), each on one line: the line breaks of a paragraph become
    /// spaces, or nothing in Japanese, which has no spaces between words.
    pub(crate) fn paragraphs(self, text: &str) -> Vec<String> {
        let joint = if self.0 == Language::Japanese {
            ""
        } else {
            " "
        };
        text.split("\n\n")
            .map(|paragraph| paragraph.trim().lines().collect::<Vec<_>>().join(joint))
            .filter(|paragraph| !paragraph.is_empty())
            .collect()
    }
}

fn english_month(number: u8) -> &'static str {
    ENGLISH_MONTHS
        .get(usize::from(number) - 1)
        .copied()
        .unwrap_or_default()
}

fn japanese_duration(months: u32) -> String {
    let (years, rest) = (months / 12, months % 12);
    match (years, rest) {
        (0, rest) => format!("{rest}ヶ月"),
        (years, 0) => format!("{years}年"),
        (years, rest) => format!("{years}年{rest}ヶ月"),
    }
}

#[cfg(test)]
mod tests {
    use kariera_domain::Language;

    use super::Locale;
    use crate::test_career::month;

    const ENGLISH: Locale = Locale(Language::English);
    const JAPANESE: Locale = Locale(Language::Japanese);
    const GERMAN: Locale = Locale(Language::German);

    #[test]
    fn each_language_writes_a_month_its_way() {
        let october = month("2023-10");

        assert_eq!(ENGLISH.month(october), "Oct 2023");
        assert_eq!(GERMAN.month(october), "10/2023");
        assert_eq!(JAPANESE.month(october), "2023年10月");
    }

    #[test]
    fn each_language_writes_a_date_its_way() {
        let date = "2026-04-01".parse().unwrap();

        assert_eq!(ENGLISH.date(date), "1 Apr 2026");
        assert_eq!(GERMAN.date(date), "01.04.2026");
        assert_eq!(JAPANESE.date(date), "2026年4月1日");
    }

    #[test]
    fn a_finished_period_runs_from_its_start_to_its_end() {
        let (start, end) = (month("2021-04"), Some(month("2023-09")));

        assert_eq!(ENGLISH.period(start, end), "Apr 2021 – Sep 2023");
        assert_eq!(GERMAN.period(start, end), "04/2021 – 09/2023");
        assert_eq!(JAPANESE.period(start, end), "2021年4月～2023年9月");
    }

    #[test]
    fn a_running_period_says_since_when() {
        let start = month("2023-10");

        assert_eq!(ENGLISH.period(start, None), "since Oct 2023");
        assert_eq!(GERMAN.period(start, None), "seit 10/2023");
        assert_eq!(JAPANESE.period(start, None), "2023年10月～現在");
    }

    #[test]
    fn a_period_of_one_month_names_it_once() {
        let only = month("2024-02");

        assert_eq!(ENGLISH.period(only, Some(only)), "Feb 2024");
        assert_eq!(JAPANESE.period(only, Some(only)), "2024年2月");
    }

    #[test]
    fn experience_is_years_with_one_decimal_in_english_and_german() {
        assert_eq!(ENGLISH.experience(50), "4.2 y");
        assert_eq!(GERMAN.experience(50), "4,2 J.");
        assert_eq!(ENGLISH.experience(36), "3.0 y");
        assert_eq!(ENGLISH.experience(1), "0.1 y");
    }

    #[test]
    fn experience_is_years_and_months_in_japanese() {
        assert_eq!(JAPANESE.experience(46), "3年10ヶ月");
        assert_eq!(JAPANESE.experience(36), "3年");
        assert_eq!(JAPANESE.experience(7), "7ヶ月");
    }

    #[test]
    fn a_range_of_days_uses_the_language_s_dash() {
        assert_eq!(ENGLISH.range(3, 5), "3–5");
        assert_eq!(JAPANESE.range(3, 5), "3～5");
        assert_eq!(GERMAN.range(4, 4), "4");
    }

    #[test]
    fn japanese_joins_the_lines_of_a_paragraph_without_a_space() {
        assert_eq!(
            JAPANESE.paragraphs("一行目\n二行目\n\n次の段落"),
            ["一行目二行目", "次の段落"]
        );
        assert_eq!(
            ENGLISH.paragraphs("First line\nsecond line\n\n\nNext"),
            ["First line second line", "Next"]
        );
        assert_eq!(ENGLISH.paragraphs(""), Vec::<String>::new());
    }
}
