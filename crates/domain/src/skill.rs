use crate::fixed_list::labelled_list;
use crate::{Fingerprint, Month, Source, Text, Translated, Translations};

/// Schema §2.6. `level` is set only for a `human-language`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    pub name: Text,
    pub aliases: Vec<Text>,
    pub category: SkillCategory,
    /// Empty where `name` is used as it is.
    pub labels: Translations<Text>,
    pub level: Option<Translated<Text>>,
    /// The owner's most recent use outside any Project. It never adds to computed years of experience; it only
    /// decides whether the Skill counts as recently used.
    pub last_used: Option<Month>,
    pub translated_from: Translations<Fingerprint>,
    pub sources: Vec<Source>,
    pub reviewed: bool,
}

labelled_list!(pub SkillCategory {
    Language => "language" ["Programming language", "プログラミング言語", "Programmiersprache"],
    Framework => "framework" ["Framework", "フレームワーク", "Framework"],
    Library => "library" ["Library", "ライブラリ", "Bibliothek"],
    Database => "database" ["Database", "データベース", "Datenbank"],
    Cloud => "cloud" ["Cloud", "クラウド", "Cloud"],
    Platform => "platform" ["Platform", "プラットフォーム", "Plattform"],
    Tool => "tool" ["Tool", "ツール", "Werkzeug"],
    AiTool => "ai-tool" ["AI tool", "AIツール", "KI-Werkzeug"],
    Method => "method" ["Method", "手法", "Methode"],
    HumanLanguage => "human-language" ["Human language", "語学", "Sprache"],
});
