use crate::fixed_list::labelled_list;
use crate::{
    Bullets, Engagement, Fingerprint, Link, Month, Organization, Paragraphs, Skill, Source,
    TeamSize, Text, Translated, Translations,
};

/// Schema §2.4. `public_title` is set exactly when `confidential` is (§1.9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub engagement: Link<Engagement>,
    pub end_client: Option<Link<Organization>>,
    pub title: Translated<Text>,
    pub public_title: Option<Translated<Text>>,
    pub confidential: bool,
    pub aliases: Vec<Text>,
    pub start: Month,
    /// Empty while the Project is ongoing.
    pub end: Option<Month>,
    pub role: Translated<Text>,
    pub focus: Option<Translated<Text>>,
    pub team_size: Option<TeamSize>,
    pub industry: Translated<Text>,
    pub phases: Vec<Phase>,
    /// At least one.
    pub skills: Vec<Link<Skill>>,
    pub translated_from: Translations<Fingerprint>,
    pub sources: Vec<Source>,
    pub reviewed: bool,
    pub description: Translated<Paragraphs>,
    pub achievements: Translated<Bullets>,
}

labelled_list!(pub Phase {
    Requirements => "requirements" ["Requirements", "要件定義", "Anforderungsdefinition"],
    BasicDesign => "basic-design" ["Basic design", "基本設計", "Grobentwurf"],
    DetailedDesign => "detailed-design" ["Detailed design", "詳細設計", "Detailentwurf"],
    Implementation => "implementation" ["Implementation", "製造", "Implementierung"],
    Testing => "testing" ["Testing", "テスト", "Test"],
    Operations => "operations" ["Operations", "運用・保守", "Betrieb und Wartung"],
});
