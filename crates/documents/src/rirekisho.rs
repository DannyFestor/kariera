//! The 履歴書, Japanese: layout variant A「厚生労働省様式」 (issue #22, branch `prototype/rirekisho`).

mod history;

use kariera_domain::{Date, Personal, Photo, Text};
use serde::Serialize;

use crate::context::Context;
use crate::labels::RirekishoLabels;
use crate::locale::Locale;
use crate::parts::SpokenLanguage;

pub use history::HistoryLine;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Rirekisho {
    pub labels: RirekishoLabels,
    pub name: String,
    pub name_kana: Option<String>,
    /// 作成日, as the date it states: 「2026年11月25日現在」.
    pub as_of: String,
    pub gender: Option<String>,
    /// 「1990年1月1日生」.
    pub birth_date: Option<String>,
    /// 満年齢 at the build date: 「満36歳」.
    pub age: Option<String>,
    /// The rows of a foreign applicant, left out when none of their values is set.
    pub residence: Option<Residence>,
    pub address: Address,
    pub phone: Option<String>,
    pub email: Option<String>,
    #[serde(skip)]
    pub photo: Option<Photo>,
    pub education: Vec<HistoryLine>,
    pub career: Vec<HistoryLine>,
    pub licences: Vec<HistoryLine>,
    /// The Self-PR, then the human languages.
    pub appeal: Vec<String>,
    pub wishes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Residence {
    pub nationality: Option<String>,
    /// The 在留資格, with the work restriction in brackets.
    pub status: Option<String>,
    pub expiry: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Address {
    pub postal_code: Option<String>,
    pub text: Option<String>,
    pub kana: Option<String>,
}

pub(crate) fn build(context: &Context<'_>) -> Rirekisho {
    let (profile_name, profile) = context.profile;
    let personal = context.personal;
    let locale = context.locale;
    let mut appeal = context.paragraphs(profile_name, &profile.self_pr);
    appeal.extend(languages_line(&context.spoken_languages()));
    Rirekisho {
        labels: RirekishoLabels::JAPANESE,
        name: profile.name.to_string(),
        name_kana: text(personal.name_kana.as_ref()),
        as_of: format!("{}現在", locale.date(context.today)),
        gender: text(personal.gender.as_ref()),
        birth_date: personal
            .birth_date
            .map(|birth| format!("{}生", locale.date(birth))),
        age: personal
            .birth_date
            .map(|birth| format!("満{}歳", age(birth, context.today))),
        residence: residence(personal, locale),
        address: Address {
            postal_code: text(personal.postal_code.as_ref()),
            text: text(personal.address.as_ref()),
            kana: text(personal.address_kana.as_ref()),
        },
        phone: text(personal.phone.as_ref()),
        email: text(personal.email.as_ref()),
        photo: personal.photo.clone(),
        education: history::education(context),
        career: history::career(context),
        licences: history::licences(context),
        appeal,
        wishes: wishes(
            profile.available_from.map(|from| locale.date(from)),
            context.days_per_week(),
        ),
    }
}

fn text(value: Option<&Text>) -> Option<String> {
    value.map(ToString::to_string)
}

/// Whole years since `birth`, the birthday itself counted.
fn age(birth: Date, today: Date) -> u16 {
    let birthday_to_come =
        (today.month().number(), today.day()) < (birth.month().number(), birth.day());
    today
        .month()
        .year()
        .saturating_sub(birth.month().year())
        .saturating_sub(u16::from(birthday_to_come))
}

fn residence(personal: &Personal, locale: Locale) -> Option<Residence> {
    let residence = Residence {
        nationality: personal
            .nationality
            .as_ref()
            .map(|nationality| nationality.get_or_english(locale.0).to_string()),
        status: status(
            personal.residence_status.as_ref(),
            personal.work_restriction.as_ref(),
        ),
        expiry: personal.residence_expiry.map(|expiry| locale.date(expiry)),
        note: text(personal.residence_note.as_ref()),
    };
    let is_set = residence.nationality.is_some()
        || personal.residence_status.is_some()
        || residence.expiry.is_some()
        || residence.note.is_some();
    is_set.then_some(residence)
}

fn status(status: Option<&Text>, restriction: Option<&Text>) -> Option<String> {
    match (status, restriction) {
        (Some(status), Some(restriction)) => Some(format!("{status}（{restriction}）")),
        (Some(status), None) => Some(status.to_string()),
        (None, Some(restriction)) => Some(format!("（{restriction}）")),
        (None, None) => None,
    }
}

fn languages_line(languages: &[SpokenLanguage]) -> Option<String> {
    let languages: Vec<String> = languages
        .iter()
        .map(|language| {
            language.level.as_ref().map_or_else(
                || language.name.clone(),
                |level| format!("{}（{level}）", language.name),
            )
        })
        .collect();
    (!languages.is_empty()).then(|| format!("語学：{}", languages.join("、")))
}

/// `days_per_week` is a range such as 「3～5」.
fn wishes(available_from: Option<String>, days_per_week: Option<String>) -> Vec<String> {
    let wishes: Vec<String> = available_from
        .map(|from| format!("勤務開始可能日：{from}"))
        .into_iter()
        .chain(days_per_week.map(|days| format!("希望勤務日数：週{days}日")))
        .collect();
    let rules = if wishes.is_empty() {
        "貴社の規定に従います。"
    } else {
        "その他は貴社の規定に従います。"
    };
    wishes.into_iter().chain([rules.to_owned()]).collect()
}

#[cfg(test)]
mod tests {
    use kariera_domain::{Language, Personal, Translated};

    use super::{age, languages_line, residence, status, wishes};
    use crate::locale::Locale;
    use crate::parts::SpokenLanguage;
    use crate::test_career::text;

    const JAPANESE: Locale = Locale(Language::Japanese);

    #[test]
    fn the_age_counts_a_year_from_each_birthday_on() {
        let birth = "1990-06-15".parse().unwrap();

        assert_eq!(age(birth, "2026-06-14".parse().unwrap()), 35);
        assert_eq!(age(birth, "2026-06-15".parse().unwrap()), 36);
        assert_eq!(age(birth, "2026-11-25".parse().unwrap()), 36);
    }

    #[test]
    fn the_residence_rows_are_left_out_when_none_of_their_values_is_set() {
        let personal = Personal {
            work_restriction: Some(text("就労制限なし")),
            ..Personal::default()
        };

        assert_eq!(residence(&personal, JAPANESE), None);
    }

    #[test]
    fn the_nationality_is_the_japanese_one_else_the_english_one() {
        let nationality = |ja: Option<&str>| Personal {
            nationality: Some(Translated {
                en: text("German"),
                ja: ja.map(text),
                de: None,
            }),
            ..Personal::default()
        };

        let japanese = residence(&nationality(Some("ドイツ")), JAPANESE).unwrap();
        let english = residence(&nationality(None), JAPANESE).unwrap();

        assert_eq!(japanese.nationality.as_deref(), Some("ドイツ"));
        assert_eq!(english.nationality.as_deref(), Some("German"));
        assert_eq!(english.status, None);
    }

    #[test]
    fn the_work_restriction_follows_the_status_in_brackets() {
        let (status_text, restriction) = (text("技術・人文知識・国際業務"), text("就労制限なし"));

        assert_eq!(
            status(Some(&status_text), Some(&restriction)).as_deref(),
            Some("技術・人文知識・国際業務（就労制限なし）")
        );
        assert_eq!(
            status(Some(&status_text), None).as_deref(),
            Some("技術・人文知識・国際業務")
        );
        assert_eq!(status(None, None), None);
    }

    #[test]
    fn the_languages_line_names_each_language_with_its_level() {
        let languages = [
            SpokenLanguage {
                name: "ドイツ語".to_owned(),
                level: Some("ネイティブ".to_owned()),
            },
            SpokenLanguage {
                name: "英語".to_owned(),
                level: None,
            },
        ];

        assert_eq!(
            languages_line(&languages).as_deref(),
            Some("語学：ドイツ語（ネイティブ）、英語")
        );
        assert_eq!(languages_line(&[]), None);
    }

    #[test]
    fn the_wishes_state_the_availability_then_defer_to_the_company() {
        assert_eq!(
            wishes(Some("2026年4月1日".to_owned()), Some("3～5".to_owned())),
            [
                "勤務開始可能日：2026年4月1日",
                "希望勤務日数：週3～5日",
                "その他は貴社の規定に従います。"
            ]
        );
        assert_eq!(
            wishes(None, Some("4".to_owned())),
            ["希望勤務日数：週4日", "その他は貴社の規定に従います。"]
        );
        assert_eq!(wishes(None, None), ["貴社の規定に従います。"]);
    }
}
