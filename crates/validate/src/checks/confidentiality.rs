//! Schema §1.9: no note but the End Client's own mentions a confidential Project or its End Client in prose.

use kariera_domain::{Organization, Project};
use kariera_vault::{NoteFile, nfc};

use crate::body_lines::body_lines;
use crate::context::{Context, Found};
use crate::field_path::FieldPath;
use crate::finding::{At, Finding, Rule};
use crate::sourced::{SourceOf, all_sources};

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    let secrets: Vec<Secret> = context
        .notes::<Project>()
        .filter(|project| project.note.confidential)
        .flat_map(|project| secrets_of(context, &project))
        .collect();
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

/// A word that must not appear in prose, and what it gives away.
struct Secret {
    needle: String,
    gives_away: String,
    /// The End Client's own Organization note may name it.
    exempt: Option<kariera_domain::NoteName>,
}

fn secrets_of(context: &Context<'_>, project: &Found<'_, Project>) -> Vec<Secret> {
    let exempt = project
        .note
        .end_client
        .as_ref()
        .map(|link| link.target().clone());
    let titles = [
        Some(&project.note.title.en),
        project.note.title.ja.as_ref(),
        project.note.title.de.as_ref(),
    ];
    let title_secrets = titles.into_iter().flatten().map(|title| Secret {
        needle: lowercase(title.as_str()),
        gives_away: format!("the title of the confidential Project [[{}]]", project.name),
        exempt: exempt.clone(),
    });
    let end_client_secrets = project
        .note
        .end_client
        .as_ref()
        .and_then(|link| context.get(link))
        .into_iter()
        .flat_map(|organization| names_of(&organization))
        .map(|name| Secret {
            needle: lowercase(&name),
            gives_away: format!(
                "the End Client of the confidential Project [[{}]]",
                project.name
            ),
            exempt: exempt.clone(),
        });
    title_secrets.chain(end_client_secrets).collect()
}

fn names_of(organization: &Found<'_, Organization>) -> Vec<String> {
    let aliases = organization.note.aliases.iter().map(ToString::to_string);
    std::iter::once(organization.note.name.to_string())
        .chain(aliases)
        .collect()
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
}
