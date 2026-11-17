use crate::{
    Bullets, Date, DaysPerWeek, Fingerprint, Link, Organization, Paragraphs, Source, Text,
    Translated, Translations, Url,
};

/// Schema §2.8. Personal data never appears here: it lives outside the vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub name: Text,
    pub headline: Translated<Text>,
    pub business: Link<Organization>,
    /// Empty when there is no availability statement.
    pub available_from: Option<Date>,
    pub days_per_week_min: Option<DaysPerWeek>,
    pub days_per_week_max: Option<DaysPerWeek>,
    pub portfolio_url: Option<Url>,
    pub github_url: Option<Url>,
    pub translated_from: Translations<Fingerprint>,
    pub sources: Vec<Source>,
    pub reviewed: bool,
    pub summary: Translated<Paragraphs>,
    pub strengths: Translated<Bullets>,
    pub self_pr: Translated<Paragraphs>,
}
