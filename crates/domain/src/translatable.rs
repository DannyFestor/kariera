use crate::{
    Bullets, Career, Engagement, Fingerprint, NoteName, NoteType, Organization, Paragraphs,
    Profile, Project, Qualification, Skill, Text, Translated, Translations,
};

/// One piece of a note's English translatable content (schema §1.6.1), with the field or section it is written in:
/// a frontmatter key such as `role`, or a body heading such as `## Description`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnglishText {
    pub part: &'static str,
    pub text: String,
}

/// The note types that carry `translated_from` (schema §1.6.1).
pub trait Translatable {
    fn translated_from(&self) -> &Translations<Fingerprint>;

    /// In the order of the type's schema table: the frontmatter fields, then the body sections. An empty optional
    /// field is left out.
    fn english_content(&self) -> Vec<EnglishText>;
}

impl Career {
    /// `None` for a note of a type that has no `translated_from`, or no such note.
    #[must_use]
    pub fn translatable(&self, note_type: NoteType, name: &NoteName) -> Option<&dyn Translatable> {
        match note_type {
            NoteType::Organization => self.organizations.get(name).map(as_translatable),
            NoteType::Engagement => self.engagements.get(name).map(as_translatable),
            NoteType::Project => self.projects.get(name).map(as_translatable),
            NoteType::Skill => self.skills.get(name).map(as_translatable),
            NoteType::Qualification => self.qualifications.get(name).map(as_translatable),
            NoteType::Profile => self.profiles.get(name).map(as_translatable),
            NoteType::Contract | NoteType::MonthlyLog => None,
        }
    }
}

fn as_translatable<T: Translatable>(note: &T) -> &dyn Translatable {
    note
}

#[derive(Default)]
struct Content(Vec<EnglishText>);

impl Content {
    fn text(mut self, part: &'static str, text: &Text) -> Self {
        self.0.push(EnglishText {
            part,
            text: text.to_string(),
        });
        self
    }

    fn english(self, part: &'static str, map3: &Translated<Text>) -> Self {
        self.text(part, &map3.en)
    }

    fn optional_english(self, part: &'static str, map3: Option<&Translated<Text>>) -> Self {
        match map3 {
            Some(map3) => self.english(part, map3),
            None => self,
        }
    }

    fn paragraphs(mut self, heading: &'static str, section: &Translated<Paragraphs>) -> Self {
        self.0.push(EnglishText {
            part: heading,
            text: section.en.0.clone(),
        });
        self
    }

    fn bullets(mut self, heading: &'static str, section: &Translated<Bullets>) -> Self {
        let lines: Vec<String> = section
            .en
            .iter()
            .map(|bullet| format!("- {bullet}"))
            .collect();
        self.0.push(EnglishText {
            part: heading,
            text: lines.join("\n"),
        });
        self
    }

    fn finish(self) -> Vec<EnglishText> {
        self.0
    }
}

impl Translatable for Organization {
    fn translated_from(&self) -> &Translations<Fingerprint> {
        &self.translated_from
    }

    fn english_content(&self) -> Vec<EnglishText> {
        Content::default()
            .paragraphs("## Description", &self.description)
            .finish()
    }
}

impl Translatable for Engagement {
    fn translated_from(&self) -> &Translations<Fingerprint> {
        &self.translated_from
    }

    fn english_content(&self) -> Vec<EnglishText> {
        Content::default()
            .optional_english("role", self.role.as_ref())
            .paragraphs("## Summary", &self.summary)
            .finish()
    }
}

impl Translatable for Project {
    fn translated_from(&self) -> &Translations<Fingerprint> {
        &self.translated_from
    }

    fn english_content(&self) -> Vec<EnglishText> {
        Content::default()
            .english("title", &self.title)
            .optional_english("public_title", self.public_title.as_ref())
            .english("role", &self.role)
            .optional_english("focus", self.focus.as_ref())
            .english("industry", &self.industry)
            .paragraphs("## Description", &self.description)
            .bullets("## Achievements", &self.achievements)
            .finish()
    }
}

impl Translatable for Skill {
    fn translated_from(&self) -> &Translations<Fingerprint> {
        &self.translated_from
    }

    fn english_content(&self) -> Vec<EnglishText> {
        Content::default()
            .text("name", &self.name)
            .optional_english("level", self.level.as_ref())
            .finish()
    }
}

impl Translatable for Qualification {
    fn translated_from(&self) -> &Translations<Fingerprint> {
        &self.translated_from
    }

    fn english_content(&self) -> Vec<EnglishText> {
        Content::default().text("name", &self.name).finish()
    }
}

impl Translatable for Profile {
    fn translated_from(&self) -> &Translations<Fingerprint> {
        &self.translated_from
    }

    fn english_content(&self) -> Vec<EnglishText> {
        Content::default()
            .english("headline", &self.headline)
            .paragraphs("## Summary", &self.summary)
            .bullets("## Strengths", &self.strengths)
            .paragraphs("## Self-PR", &self.self_pr)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{EnglishText, Translatable};
    use crate::{
        Career, Engagement, EngagementType, Link, Month, NoteType, Organization, Paragraphs,
        Profile, Project, Qualification, QualificationCategory, Skill, SkillCategory, Text,
        Translated, Translations,
    };

    fn text(value: &str) -> Text {
        value.parse().unwrap()
    }

    fn map3(en: &str) -> Translated<Text> {
        Translated {
            en: text(en),
            ja: Some(text("日本語")),
            de: Some(text("Deutsch")),
        }
    }

    fn paragraphs(en: &str) -> Translated<Paragraphs> {
        Translated {
            en: Paragraphs(en.to_owned()),
            ja: Some(Paragraphs("日本語".to_owned())),
            de: None,
        }
    }

    fn bullets(en: &[&str]) -> Translated<Vec<Text>> {
        Translated {
            en: en.iter().map(|bullet| text(bullet)).collect(),
            ja: Some(vec![text("日本語")]),
            de: None,
        }
    }

    fn month() -> Month {
        "2025-06".parse().unwrap()
    }

    fn content(note: &dyn Translatable) -> Vec<(&'static str, String)> {
        note.english_content()
            .into_iter()
            .map(|EnglishText { part, text }| (part, text))
            .collect()
    }

    fn pairs(parts: &[(&'static str, &str)]) -> Vec<(&'static str, String)> {
        parts
            .iter()
            .map(|&(part, text)| (part, text.to_owned()))
            .collect()
    }

    fn organization() -> Organization {
        Organization {
            name: text("Acme"),
            aliases: Vec::new(),
            kind: Vec::new(),
            location: None,
            website: None,
            translated_from: Translations::none(),
            sources: Vec::new(),
            reviewed: false,
            description: paragraphs("Makes things.\n\nSells them."),
        }
    }

    fn engagement(role: Option<Translated<Text>>) -> Engagement {
        Engagement {
            client: None,
            employer: None,
            engagement_type: EngagementType::Freelance,
            role,
            start: month(),
            end: None,
            contracts: Vec::new(),
            translated_from: Translations::none(),
            sources: Vec::new(),
            reviewed: false,
            summary: paragraphs("Built it."),
        }
    }

    fn project() -> Project {
        Project {
            engagement: Link::new("2025-06 Acme".parse().unwrap()),
            end_client: None,
            title: map3("Shop"),
            public_title: Some(map3("Online shop")),
            confidential: true,
            aliases: Vec::new(),
            start: month(),
            end: None,
            role: map3("Developer"),
            focus: Some(map3("Backend")),
            team_size: None,
            industry: map3("Retail"),
            phases: Vec::new(),
            skills: Vec::new(),
            translated_from: Translations::none(),
            sources: Vec::new(),
            reviewed: false,
            description: paragraphs("Built the shop."),
            achievements: bullets(&["Shipped", "Scaled"]),
        }
    }

    fn skill(level: Option<Translated<Text>>) -> Skill {
        Skill {
            name: text("Japanese"),
            aliases: Vec::new(),
            category: SkillCategory::HumanLanguage,
            labels: Translations::none(),
            level,
            translated_from: Translations::none(),
            sources: Vec::new(),
            reviewed: false,
        }
    }

    fn qualification() -> Qualification {
        Qualification {
            name: text("Cloud Associate"),
            labels: Translations::none(),
            category: QualificationCategory::Certificate,
            issuer: Some(text("Example Cloud")),
            start: None,
            date: month(),
            level: Some(text("Associate")),
            score: Some(text("900")),
            translated_from: Translations::none(),
            sources: Vec::new(),
            reviewed: false,
        }
    }

    fn profile() -> Profile {
        Profile {
            name: text("Robin Beispiel"),
            headline: map3("Fullstack developer"),
            business: Link::new("Example Works".parse().unwrap()),
            available_from: None,
            days_per_week_min: None,
            days_per_week_max: None,
            portfolio_url: None,
            github_url: None,
            translated_from: Translations::none(),
            sources: Vec::new(),
            reviewed: false,
            summary: paragraphs("Builds things."),
            strengths: bullets(&["Go", "SQL"]),
            self_pr: paragraphs("Works alone."),
        }
    }

    #[test]
    fn an_organization_is_its_english_description() {
        assert_eq!(
            content(&organization()),
            pairs(&[("## Description", "Makes things.\n\nSells them.")])
        );
    }

    #[test]
    fn an_engagement_is_its_role_and_english_summary() {
        assert_eq!(
            content(&engagement(Some(map3("Lead")))),
            pairs(&[("role", "Lead"), ("## Summary", "Built it.")])
        );
        assert_eq!(
            content(&engagement(None)),
            pairs(&[("## Summary", "Built it.")])
        );
    }

    #[test]
    fn a_project_is_its_english_fields_and_sections() {
        assert_eq!(
            content(&project()),
            pairs(&[
                ("title", "Shop"),
                ("public_title", "Online shop"),
                ("role", "Developer"),
                ("focus", "Backend"),
                ("industry", "Retail"),
                ("## Description", "Built the shop."),
                ("## Achievements", "- Shipped\n- Scaled"),
            ])
        );
    }

    #[test]
    fn a_project_without_optional_fields_leaves_them_out() {
        let project = Project {
            public_title: None,
            focus: None,
            ..project()
        };

        let parts: Vec<_> = content(&project)
            .into_iter()
            .map(|(part, _)| part)
            .collect();

        assert_eq!(
            parts,
            [
                "title",
                "role",
                "industry",
                "## Description",
                "## Achievements"
            ]
        );
    }

    #[test]
    fn a_skill_is_its_name_and_english_level() {
        assert_eq!(
            content(&skill(Some(map3("Fluent")))),
            pairs(&[("name", "Japanese"), ("level", "Fluent")])
        );
        assert_eq!(content(&skill(None)), pairs(&[("name", "Japanese")]));
    }

    #[test]
    fn a_qualification_is_its_name_only() {
        assert_eq!(
            content(&qualification()),
            pairs(&[("name", "Cloud Associate")])
        );
    }

    #[test]
    fn a_profile_is_its_headline_and_english_sections() {
        assert_eq!(
            content(&profile()),
            pairs(&[
                ("headline", "Fullstack developer"),
                ("## Summary", "Builds things."),
                ("## Strengths", "- Go\n- SQL"),
                ("## Self-PR", "Works alone."),
            ])
        );
    }

    #[test]
    fn the_career_finds_the_translatable_note_of_a_name_and_type() {
        let mut career = Career::default();
        let name = "Acme".parse().unwrap();
        career
            .organizations
            .insert("Acme".parse().unwrap(), organization());

        let found = career.translatable(NoteType::Organization, &name);

        assert_eq!(found.map(|note| note.english_content().len()), Some(1));
        assert!(career.translatable(NoteType::Skill, &name).is_none());
        assert!(career.translatable(NoteType::Contract, &name).is_none());
        assert!(career.translatable(NoteType::MonthlyLog, &name).is_none());
    }
}
