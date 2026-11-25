use crate::fixed_list::labelled_list;
use crate::{Fingerprint, Link, Month, Organization, Source, Text, Translations};

/// Schema §2.7. A degree's `issuer` is a `School`, every other category's a `Body` (kept by the vault reader).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Qualification {
    pub name: Text,
    /// Empty where `name` is used as it is.
    pub labels: Translations<Text>,
    pub category: QualificationCategory,
    pub issuer: Option<Issuer>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Issuer {
    /// An Organization whose `kind` includes `school`, so its name can be printed in the document's language.
    School(Link<Organization>),
    /// The issuing body's official form, a proper noun that is never translated (schema §1.5).
    Body(Text),
}

labelled_list!(pub QualificationCategory {
    Degree => "degree" ["Degree or school diploma (listed as education)", "学歴", "Abschluss"],
    Certificate => "certificate" ["Certificate", "資格", "Zertifikat"],
    Licence => "licence" ["Licence", "免許", "Lizenz"],
    TestResult => "test-result" ["Test result", "試験結果", "Testergebnis"],
});
