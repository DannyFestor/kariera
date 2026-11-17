use crate::fixed_list::labelled_list;
use crate::{
    Contract, Link, Month, Organization, Paragraphs, Source, Text, Translated, Translations,
};

/// Schema §2.2. Exactly one of `client` and `employer` is set, the one `engagement_type` asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Engagement {
    pub client: Option<Link<Organization>>,
    pub employer: Option<Link<Organization>>,
    #[expect(
        clippy::struct_field_names,
        reason = "the fields carry the schema's key names (§2.2)"
    )]
    pub engagement_type: EngagementType,
    pub role: Option<Translated<Text>>,
    pub start: Month,
    /// Empty while the Engagement is ongoing.
    pub end: Option<Month>,
    pub contracts: Vec<Link<Contract>>,
    pub translated_from: Translations<Text>,
    pub sources: Vec<Source>,
    pub reviewed: bool,
    pub summary: Translated<Paragraphs>,
}

labelled_list!(pub EngagementType {
    Employment => "employment" ["Employment", "直接雇用", "Festanstellung"],
    Dispatch => "dispatch" ["Dispatch", "派遣", "Arbeitnehmerüberlassung"],
    Internship => "internship" ["Internship", "インターンシップ", "Praktikum"],
    Freelance => "freelance" ["Freelance", "フリーランス", "Freiberuflich"],
    OwnBusiness => "own-business" ["Own business", "個人事業", "Eigenes Unternehmen"],
});
