use crate::fixed_list::labelled_list;
use crate::{Fingerprint, Month, Source, Text, Translations};

/// Schema §2.7.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Qualification {
    pub name: Text,
    /// Empty where `name` is used as it is.
    pub labels: Translations<Text>,
    pub category: QualificationCategory,
    pub issuer: Option<Text>,
    /// Start of study, for degrees.
    pub start: Option<Month>,
    /// The month obtained, passed or graduated.
    pub date: Month,
    pub level: Option<Text>,
    pub score: Option<Text>,
    pub translated_from: Translations<Fingerprint>,
    pub sources: Vec<Source>,
    pub reviewed: bool,
}

labelled_list!(pub QualificationCategory {
    Degree => "degree" ["Degree or school diploma (listed as education)", "学歴", "Abschluss"],
    Certificate => "certificate" ["Certificate", "資格", "Zertifikat"],
    Licence => "licence" ["Licence", "免許", "Lizenz"],
    TestResult => "test-result" ["Test result", "試験結果", "Testergebnis"],
});
