//! Schema §1.9: no note but the End Client's own mentions a confidential Project or its End Client in prose, and
//! nothing kariera publishes mentions them at all.

use std::fmt;

use kariera_domain::{Career, Language, NoteName, Organization, Project, Text};
use kariera_vault::{NoteFile, nfc};

use crate::body_lines::body_lines;
use crate::context::Context;
use crate::field_path::FieldPath;
use crate::finding::{At, Finding, Rule};
use crate::sourced::{SourceOf, all_sources};

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    let secrets = secrets(context.career());
    let sources = all_sources(context);
    let mut findings = Vec::new();
    for file in context.files() {
        findings.extend(body_mentions(file, &secrets));
    }
    for cited in &sources {
        findings.extend(source_mentions(cited, &secrets));
    }
    findings
}

/// A text kariera would publish that gives a confidential Project away.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disclosure {
    pub text: String,
    pub gives_away: String,
}

impl fmt::Display for Disclosure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?} gives away {}", self.text, self.gives_away)
    }
}

/// Every published text that names a confidential Project's title or End Client. Unlike prose in the vault, no
/// text is exempt: the End Client's own note is never published.
pub fn disclosures<'text>(
    career: &Career,
    texts: impl IntoIterator<Item = &'text str>,
) -> Vec<Disclosure> {
    let secrets = secrets(career);
    texts
        .into_iter()
        .flat_map(|text| {
            let compared = lowercase(text);
            secrets
                .iter()
                .filter(move |secret| compared.contains(&secret.needle))
                .map(move |secret| Disclosure {
                    text: text.to_owned(),
                    gives_away: secret.gives_away.clone(),
                })
        })
        .collect()
}

/// A word that must not appear in prose, and what it gives away.
struct Secret {
    needle: String,
    gives_away: String,
    /// The End Client's own Organization note may name it.
    exempt: Option<NoteName>,
}

fn secrets(career: &Career) -> Vec<Secret> {
    career
        .projects
        .iter()
        .filter(|(_, project)| project.confidential)
        .flat_map(|(name, project)| secrets_of(career, name, project))
        .collect()
}

fn secrets_of(career: &Career, name: &NoteName, project: &Project) -> Vec<Secret> {
    let exempt = project
        .end_client
        .as_ref()
        .map(|link| link.target().clone());
    let title_secrets = secret_titles(project).map(|needle| Secret {
        needle,
        gives_away: format!("the title of the confidential Project [[{name}]]"),
        exempt: exempt.clone(),
    });
    let end_client_secrets = project
        .end_client
        .as_ref()
        .and_then(|link| career.get(link))
        .into_iter()
        .flat_map(names_of)
        .map(|end_client_name| Secret {
            needle: lowercase(end_client_name.as_str()),
            gives_away: format!("the End Client of the confidential Project [[{name}]]"),
            exempt: exempt.clone(),
        });
    title_secrets.chain(end_client_secrets).collect()
}

/// A title that equals the `public_title` in its language is public wording, so only the others are secret.
fn secret_titles(project: &Project) -> impl Iterator<Item = String> {
    Language::ALL.into_iter().filter_map(|language| {
        let title = lowercase(project.title.get(language)?.as_str());
        let public_title = project
            .public_title
            .as_ref()
            .and_then(|public_title| public_title.get(language))
            .map(|public_title| lowercase(public_title.as_str()));
        (public_title.as_ref() != Some(&title)).then_some(title)
    })
}

/// Its name in every language: `name`, `aliases` and `labels`.
fn names_of(organization: &Organization) -> impl Iterator<Item = &Text> {
    let labels = [
        organization.labels.ja.as_ref(),
        organization.labels.de.as_ref(),
    ];
    std::iter::once(&organization.name)
        .chain(&organization.aliases)
        .chain(labels.into_iter().flatten())
}

fn lowercase(text: &str) -> String {
    nfc(text).to_lowercase()
}

fn body_mentions(file: &NoteFile, secrets: &[Secret]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for body_line in body_lines(&file.body) {
        let text = lowercase(&body_line.line.text);
        for secret in applicable(secrets, file).filter(|secret| text.contains(&secret.needle)) {
            findings.push(Finding::new(
                file,
                Rule::Confidentiality,
                At::Section {
                    heading: body_line.heading.clone(),
                    line: body_line.line.number,
                },
                format!(
                    "mentions {}: write the `public_title` wording instead",
                    secret.gives_away
                ),
            ));
        }
    }
    findings
}

fn source_mentions(cited: &SourceOf<'_>, secrets: &[Secret]) -> Vec<Finding> {
    let Some(note) = &cited.source.note else {
        return Vec::new();
    };
    let text = lowercase(note.as_str());
    applicable(secrets, cited.file)
        .filter(|secret| text.contains(&secret.needle))
        .map(|secret| {
            Finding::new(
                cited.file,
                Rule::Confidentiality,
                At::Field(FieldPath::of("sources").item(cited.index).key("note")),
                format!(
                    "mentions {}: write the `public_title` wording instead",
                    secret.gives_away
                ),
            )
        })
        .collect()
}

fn applicable<'a>(secrets: &'a [Secret], file: &'a NoteFile) -> impl Iterator<Item = &'a Secret> {
    secrets
        .iter()
        .filter(move |secret| secret.exempt.as_ref() != Some(&file.name))
}

#[cfg(test)]
mod tests {
    use kariera_domain::Career;
    use kariera_vault::Vault;

    use super::disclosures;
    use crate::test_vault::TestVault;

    const ENGAGEMENT: &str = "Engagements/2021-04 Acme.md";
    const SUMMARY: &str = "Built backends for online shops";

    fn end_client_of(project: &str) -> String {
        format!(
            "mentions the End Client of the confidential Project [[{project}]]: write the `public_title` wording instead"
        )
    }

    #[test]
    fn the_end_client_of_a_confidential_project_is_not_named_in_prose() {
        let vault = TestVault::new("confidential_end_client");
        vault.replace(ENGAGEMENT, SUMMARY, "Built backends for Northwind");

        assert_eq!(
            vault.lines(),
            [format!(
                "Engagements/2021-04 Acme.md:23: error [confidentiality] section `## Summary` {}",
                end_client_of("Acme - Ledger")
            )]
        );
    }

    #[test]
    fn the_name_is_compared_ignoring_case() {
        let vault = TestVault::new("confidential_case");
        vault.replace(ENGAGEMENT, SUMMARY, "Built backends for NORTHWIND");

        assert_eq!(vault.report().errors(), 1);
    }

    #[test]
    fn an_alias_of_the_end_client_is_not_named_either() {
        let vault = TestVault::new("confidential_alias");
        vault.replace(ENGAGEMENT, SUMMARY, "Built backends for ノースウィンド運輸");

        assert_eq!(vault.report().errors(), 1);
    }

    #[test]
    fn the_title_of_a_confidential_project_is_not_named_in_prose() {
        let vault = TestVault::new("confidential_title");
        vault.replace(
            "Projects/Acme - Ledger.md",
            "en: Northwind freight ledger",
            "en: Freight ledger",
        );
        vault.replace(ENGAGEMENT, SUMMARY, "Built the freight ledger");

        assert_eq!(
            vault.lines(),
            [
                "Engagements/2021-04 Acme.md:23: error [confidentiality] section `## Summary` mentions the title of the confidential Project [[Acme - Ledger]]: write the `public_title` wording instead"
            ]
        );
    }

    #[test]
    fn the_title_in_any_language_counts() {
        let vault = TestVault::new("confidential_title_japanese");
        vault.replace(
            "Projects/Acme - Ledger.md",
            "ja: ノースウィンド運輸 運賃台帳",
            "ja: 運賃台帳",
        );
        vault.replace(
            "Projects/Acme - Ledger.md",
            "en: Northwind freight ledger",
            "en: Freight ledger",
        );
        vault.replace(ENGAGEMENT, SUMMARY, "運賃台帳を構築した");

        assert_eq!(vault.report().errors(), 1);
    }

    #[test]
    fn a_source_note_is_prose_too() {
        let vault = TestVault::new("confidential_source_note");
        vault.replace(
            "Projects/Acme - Shop.md",
            "note: Lists the project code SHOP",
            "note: Northwind pays the invoice",
        );

        assert_eq!(
            vault.lines(),
            [format!(
                "Projects/Acme - Shop.md:43: error [confidentiality] field `sources[0].note` {}",
                end_client_of("Acme - Ledger")
            )]
        );
    }

    #[test]
    fn the_confidential_project_itself_must_not_name_its_end_client() {
        let vault = TestVault::new("confidential_own_body");
        vault.replace(
            "Projects/Acme - Ledger.md",
            "for a logistics company.",
            "for Northwind.",
        );

        assert_eq!(vault.report().errors(), 1);
    }

    #[test]
    fn the_organization_of_the_end_client_may_name_it() {
        let vault = TestVault::new("confidential_exempt");
        vault.replace(
            "Organizations/Northwind.md",
            "Logistics company.",
            "Northwind runs the freight ledger. ノースウィンド運輸",
        );

        assert_eq!(vault.lines(), Vec::<String>::new());
    }

    #[test]
    fn the_end_client_of_a_public_project_may_be_named() {
        let vault = TestVault::new("confidential_public");
        vault.replace(ENGAGEMENT, SUMMARY, "Built backends for Contoso");

        assert_eq!(vault.lines(), Vec::<String>::new());
    }

    #[test]
    fn a_confidential_project_without_an_end_client_hides_only_its_title() {
        let vault = TestVault::new("confidential_no_end_client");
        vault.replace(
            "Projects/Acme - Ledger.md",
            "end_client: \"[[Northwind]]\"\n",
            "",
        );
        vault.replace(ENGAGEMENT, SUMMARY, "Built backends for Northwind");

        assert_eq!(vault.lines(), Vec::<String>::new());
    }

    #[test]
    fn a_label_of_the_end_client_is_not_named_either() {
        let vault = TestVault::new("confidential_label");
        vault.replace(
            "Organizations/Northwind.md",
            "  - end-client\n",
            "  - end-client\nlabels:\n  de: Nordwind Spedition\n",
        );
        vault.replace(ENGAGEMENT, SUMMARY, "Built backends for Nordwind Spedition");

        assert_eq!(vault.report().errors(), 1);
    }

    const LEDGER: &str = "Projects/Acme - Ledger.md";

    fn title_the_ledger_with_its_public_title(vault: &TestVault) {
        vault.replace(
            LEDGER,
            "en: Northwind freight ledger",
            "en: ACCOUNTING platform for a logistics company",
        );
        vault.replace(
            LEDGER,
            "ja: ノースウィンド運輸 運賃台帳",
            "ja: 物流会社向け会計プラットフォーム",
        );
    }

    #[test]
    fn a_title_equal_to_the_public_title_may_be_written() {
        let vault = TestVault::new("confidential_title_is_public");
        title_the_ledger_with_its_public_title(&vault);
        vault.replace(
            ENGAGEMENT,
            SUMMARY,
            "Built an accounting platform for a logistics company: 物流会社向け会計プラットフォーム",
        );

        assert_eq!(vault.lines(), Vec::<String>::new());
    }

    #[test]
    fn a_title_differing_from_the_public_title_in_its_language_is_still_secret() {
        let vault = TestVault::new("confidential_title_differs_in_one_language");
        vault.replace(
            LEDGER,
            "en: Northwind freight ledger",
            "en: Accounting platform for a logistics company",
        );
        vault.replace(LEDGER, "ja: ノースウィンド運輸 運賃台帳", "ja: 運賃台帳");
        vault.replace(ENGAGEMENT, SUMMARY, "運賃台帳を構築した");

        assert_eq!(vault.report().errors(), 1);
    }

    #[test]
    fn the_end_client_stays_secret_when_the_title_equals_the_public_title() {
        let vault = TestVault::new("confidential_title_is_public_end_client");
        title_the_ledger_with_its_public_title(&vault);
        vault.replace(ENGAGEMENT, SUMMARY, "Built backends for Northwind");

        assert_eq!(
            vault.lines(),
            [format!(
                "Engagements/2021-04 Acme.md:23: error [confidentiality] section `## Summary` {}",
                end_client_of("Acme - Ledger")
            )]
        );
    }

    #[test]
    fn published_text_with_a_title_equal_to_the_public_title_discloses_nothing() {
        let vault = TestVault::new("disclosure_title_is_public");
        title_the_ledger_with_its_public_title(&vault);

        assert_eq!(
            given_away(
                &career_of(&vault),
                "Accounting platform for a logistics company"
            ),
            Vec::<String>::new()
        );
    }

    fn career_of(vault: &TestVault) -> Career {
        Vault::load(vault.root()).unwrap().career().clone()
    }

    fn given_away(career: &Career, text: &str) -> Vec<String> {
        disclosures(career, [text])
            .into_iter()
            .map(|disclosure| disclosure.to_string())
            .collect()
    }

    #[test]
    fn published_text_with_a_confidential_title_discloses_it_ignoring_case() {
        let vault = TestVault::new("disclosure_title");

        assert_eq!(
            given_away(&career_of(&vault), "Acme - NORTHWIND FREIGHT LEDGER"),
            [
                "\"Acme - NORTHWIND FREIGHT LEDGER\" gives away the title of the confidential Project [[Acme - Ledger]]",
                "\"Acme - NORTHWIND FREIGHT LEDGER\" gives away the End Client of the confidential Project [[Acme - Ledger]]",
            ]
        );
    }

    #[test]
    fn published_text_with_the_title_in_another_language_discloses_it() {
        let vault = TestVault::new("disclosure_title_japanese");

        assert_eq!(
            given_away(&career_of(&vault), "運賃台帳").len(),
            0,
            "only the whole title counts"
        );
        assert!(
            given_away(&career_of(&vault), "ノースウィンド運輸 運賃台帳")
                .iter()
                .any(|line| line.contains("the title of the confidential Project")),
        );
    }

    #[test]
    fn published_text_with_a_name_alias_or_label_of_the_end_client_discloses_it() {
        let vault = TestVault::new("disclosure_end_client");
        vault.replace(
            "Organizations/Northwind.md",
            "  - end-client\n",
            "  - end-client\nlabels:\n  de: Nordwind Spedition\n",
        );
        let career = career_of(&vault);

        for text in [
            "Built for Northwind",
            "ノースウィンド運輸",
            "Nordwind Spedition",
        ] {
            assert_eq!(
                given_away(&career, text),
                [format!(
                    "{text:?} gives away the End Client of the confidential Project [[Acme - Ledger]]"
                )]
            );
        }
    }

    #[test]
    fn published_text_with_the_public_wording_discloses_nothing() {
        let vault = TestVault::new("disclosure_none");
        let career = career_of(&vault);

        let found = disclosures(
            &career,
            [
                "Acme - Ledger",
                "Accounting platform for a logistics company",
                "Contoso toy shop backend",
            ],
        );

        assert_eq!(found, []);
    }
}
