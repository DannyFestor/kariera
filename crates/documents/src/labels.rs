//! Every heading and fixed label a document prints, in its language, so each renderer prints the same words
//! without a copy of its own. The words are those of the approved layout prototypes.

use kariera_domain::Language;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CvLabels {
    pub title: &'static str,
    pub profile: &'static str,
    pub strengths: &'static str,
    pub selected_projects: &'static str,
    pub career: &'static str,
    pub skills: &'static str,
    pub education: &'static str,
    pub certificates: &'static str,
    pub born: &'static str,
    pub client: &'static str,
    pub end_client: &'static str,
}

const ENGLISH_CV: CvLabels = CvLabels {
    title: "CV",
    profile: "Profile",
    strengths: "Strengths",
    selected_projects: "Selected projects",
    career: "Career",
    skills: "Skills",
    education: "Education",
    certificates: "Certificates and licences",
    born: "Born",
    client: "Client",
    end_client: "End client",
};

const GERMAN_CV: CvLabels = CvLabels {
    title: "Lebenslauf",
    profile: "Profil",
    strengths: "Stärken",
    selected_projects: "Ausgewählte Projekte",
    career: "Werdegang",
    skills: "IT-Kenntnisse",
    education: "Ausbildung",
    certificates: "Zertifikate und Lizenzen",
    born: "Geburtsdatum",
    client: "Kunde",
    end_client: "Endkunde",
};

impl CvLabels {
    /// The CV is written in English and German only (`DocumentKind::languages`), so Japanese never asks.
    pub(crate) const fn of(language: Language) -> Self {
        match language {
            Language::German => GERMAN_CV,
            Language::English | Language::Japanese => ENGLISH_CV,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ShokumuKeirekishoLabels {
    pub title: &'static str,
    pub summary: &'static str,
    pub strengths: &'static str,
    pub qualifications: &'static str,
    pub language: &'static str,
    pub skills: &'static str,
    pub history: &'static str,
    pub position: &'static str,
    pub business: &'static str,
    pub role: &'static str,
    pub team: &'static str,
    pub end_client: &'static str,
    pub phases: &'static str,
    pub environment: &'static str,
    pub achievements: &'static str,
    pub self_pr: &'static str,
    pub portfolio: &'static str,
    pub github: &'static str,
    pub closing: &'static str,
}

impl ShokumuKeirekishoLabels {
    pub(crate) const JAPANESE: Self = Self {
        title: "職務経歴書",
        summary: "職務要約",
        strengths: "得意分野",
        qualifications: "資格・語学",
        language: "語学",
        skills: "テクニカルスキル",
        history: "職務経歴",
        position: "役職",
        business: "事業内容",
        role: "役割",
        team: "体制",
        end_client: "エンドクライアント",
        phases: "担当工程",
        environment: "環境",
        achievements: "実績",
        self_pr: "自己PR",
        portfolio: "ポートフォリオ",
        github: "GitHub",
        closing: "以上",
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SkillSheetLabels {
    pub title: &'static str,
    pub updated: &'static str,
    pub headline: &'static str,
    pub business: &'static str,
    pub availability: &'static str,
    pub url: &'static str,
    pub certificates: &'static str,
    pub education: &'static str,
    pub strengths: &'static str,
    pub category: &'static str,
    pub skill: &'static str,
    pub experience: &'static str,
    pub last_use: &'static str,
    pub self_pr: &'static str,
    pub number: &'static str,
    pub company: &'static str,
    pub industry: &'static str,
    pub contract: &'static str,
    pub role: &'static str,
    pub team: &'static str,
    pub description: &'static str,
    pub achievements: &'static str,
    /// Printed before a phase the Project covered, where the fill alone would not survive an edit or a black and
    /// white print.
    pub done_mark: &'static str,
    /// Printed where a value is unknown, such as a team size not recorded.
    pub unknown: &'static str,
}

impl SkillSheetLabels {
    pub(crate) const JAPANESE: Self = Self {
        title: "スキルシート",
        updated: "更新日",
        headline: "得意分野",
        business: "所属",
        availability: "稼働",
        url: "URL",
        certificates: "資格",
        education: "学歴",
        strengths: "得意技術",
        category: "分類",
        skill: "スキル",
        experience: "経験",
        last_use: "最終使用",
        self_pr: "自己PR",
        number: "No.",
        company: "会社",
        industry: "業種",
        contract: "契約",
        role: "役割",
        team: "体制",
        description: "業務内容",
        achievements: "実績",
        done_mark: "●",
        unknown: "—",
    };
}
