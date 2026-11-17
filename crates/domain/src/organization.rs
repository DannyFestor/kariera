use crate::fixed_list::labelled_list;
use crate::{Paragraphs, Source, Text, Translated, Translations, Url};

/// Schema §2.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Organization {
    pub name: Text,
    pub aliases: Vec<Text>,
    /// At least one.
    pub kind: Vec<OrganizationKind>,
    pub location: Option<Text>,
    pub website: Option<Url>,
    pub translated_from: Translations<Text>,
    pub sources: Vec<Source>,
    pub reviewed: bool,
    pub description: Translated<Paragraphs>,
}

labelled_list!(pub OrganizationKind {
    MyBusiness => "my-business" ["My Business", "屋号", "Eigener Betrieb"],
    Client => "client" ["Client", "クライアント", "Kunde"],
    Agency => "agency" ["Agency", "仲介会社", "Vermittlungsagentur"],
    EndClient => "end-client" ["End Client", "エンドクライアント", "Endkunde"],
    Employer => "employer" ["Employer", "雇用主", "Arbeitgeber"],
    School => "school" ["School", "教育機関", "Bildungseinrichtung"],
});
