//! Schema §5: no pull request URLs in a note. Ticket IDs, commit hashes and personal data have no pattern that
//! finds them without also finding legitimate text, so only the URL is checked.

use crate::body_lines::body_lines;
use crate::context::Context;
use crate::field_path::FieldPath;
use crate::finding::{At, Finding, Rule};
use crate::sourced::all_sources;

const MESSAGE: &str = "contains a pull request URL, which a note must not";

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    let in_bodies = context.files().iter().flat_map(|file| {
        body_lines(&file.body)
            .into_iter()
            .filter(|body_line| mentions_pull_request(&body_line.line.text))
            .map(|body_line| {
                let at = At::Section {
                    heading: body_line.heading,
                    line: body_line.line.number,
                };
                Finding::new(file, Rule::ForbiddenContent, at, MESSAGE)
            })
    });
    let in_sources = all_sources(context).into_iter().filter_map(|cited| {
        let note = cited.source.note.as_ref()?;
        mentions_pull_request(note.as_str()).then(|| {
            let at = At::Field(FieldPath::of("sources").item(cited.index).key("note"));
            Finding::new(cited.file, Rule::ForbiddenContent, at, MESSAGE)
        })
    });
    in_bodies.chain(in_sources).collect()
}

/// `github.com/<owner>/<repository>/pull/<number>`, with or without a scheme.
fn mentions_pull_request(text: &str) -> bool {
    text.split("github.com/").skip(1).any(is_pull_request_path)
}

fn is_pull_request_path(path: &str) -> bool {
    let mut segments = path.split('/');
    matches!(
        (segments.next(), segments.next(), segments.next(), segments.next()),
        (Some(owner), Some(repository), Some("pull"), Some(number))
            if !owner.is_empty()
                && !repository.is_empty()
                && number.starts_with(|c: char| c.is_ascii_digit())
    )
}

#[cfg(test)]
mod tests {
    use super::mentions_pull_request;
    use crate::test_vault::TestVault;

    #[test]
    fn a_pull_request_url_is_found_in_any_text() {
        for text in [
            "see https://github.com/example/shop/pull/42",
            "github.com/example/shop/pull/7#issuecomment-1",
            "(https://github.com/example/shop/pull/42/files)",
        ] {
            assert!(mentions_pull_request(text), "{text}");
        }
    }

    #[test]
    fn other_github_text_is_not_a_pull_request_url() {
        for text in [
            "https://github.com/example/shop",
            "https://github.com/example/shop/pulls",
            "https://github.com/example/shop/issues/4",
            "https://github.com/example/shop/pull/",
            "https://github.com/example/pull/42",
            "https://github.com//shop/pull/42",
            "a pull request of 42 lines",
        ] {
            assert!(!mentions_pull_request(text), "{text}");
        }
    }

    #[test]
    fn a_pull_request_url_in_prose_is_reported_on_its_line() {
        let vault = TestVault::new("forbidden_body");
        vault.replace(
            "Engagements/2021-04 Acme.md",
            "Built backends",
            "Built backends (https://github.com/example/shop/pull/42)",
        );

        assert_eq!(
            vault.lines(),
            [
                "Engagements/2021-04 Acme.md:23: error [forbidden-content] section `## Summary` contains a pull request URL, which a note must not"
            ]
        );
    }

    #[test]
    fn a_pull_request_url_in_a_source_note_is_reported() {
        let vault = TestVault::new("forbidden_source_note");
        vault.replace(
            "Projects/Acme - Shop.md",
            "note: Lists the project code SHOP",
            "note: github.com/example/shop/pull/42",
        );

        assert_eq!(
            vault.lines(),
            [
                "Projects/Acme - Shop.md:43: error [forbidden-content] field `sources[0].note` contains a pull request URL, which a note must not"
            ]
        );
    }
}
