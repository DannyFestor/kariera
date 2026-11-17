//! Schema §1.4 and §1.6: no wikilinks in a body, plain paragraphs, and translations with as many bullets as the
//! English.

use kariera_domain::NoteType;
use kariera_vault::{Line, NoteFile, Section};

use crate::body_lines::{body_lines, section_heading, subsection_heading};
use crate::context::Context;
use crate::finding::{At, Finding, Rule};

const JAPANESE: &str = "日本語";
const GERMAN: &str = "Deutsch";

/// Monthly Log outcomes (schema §2.5).
const OUTCOMES: std::ops::RangeInclusive<usize> = 3..=5;

pub(super) fn check(context: &Context<'_>) -> Vec<Finding> {
    context
        .files()
        .iter()
        .flat_map(|file| {
            let mut findings = wikilinks(file);
            for (title, kind) in sections_of(file.note_type) {
                findings.extend(section_findings(file, title, *kind));
            }
            findings
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Paragraphs,
    Bullets,
    OneLine,
}

/// The English sections of a note type with their content kind (schema §2).
const fn sections_of(note_type: NoteType) -> &'static [(&'static str, Kind)] {
    match note_type {
        NoteType::Organization => &[("Description", Kind::Paragraphs)],
        NoteType::Engagement => &[("Summary", Kind::Paragraphs)],
        NoteType::Project => &[
            ("Description", Kind::Paragraphs),
            ("Achievements", Kind::Bullets),
        ],
        NoteType::MonthlyLog => &[("Focus", Kind::OneLine), ("Outcomes", Kind::Bullets)],
        NoteType::Profile => &[
            ("Summary", Kind::Paragraphs),
            ("Strengths", Kind::Bullets),
            ("Self-PR", Kind::Paragraphs),
        ],
        NoteType::Contract | NoteType::Skill | NoteType::Qualification => &[],
    }
}

fn wikilinks(file: &NoteFile) -> Vec<Finding> {
    body_lines(&file.body)
        .into_iter()
        .filter(|body_line| body_line.line.text.contains("[["))
        .map(|body_line| {
            Finding::new(
                file,
                Rule::Body,
                At::Section {
                    heading: body_line.heading,
                    line: body_line.line.number,
                },
                "contains a wikilink, which a note body must not",
            )
        })
        .collect()
}

fn section_findings(file: &NoteFile, title: &str, kind: Kind) -> Vec<Finding> {
    let Some(english) = file.body.section(title) else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    if kind == Kind::Paragraphs {
        findings.extend(plain_paragraphs(file, &section_heading(english), english));
    }
    for language in [JAPANESE, GERMAN] {
        let Some((parent, translation)) = file
            .body
            .section(language)
            .and_then(|parent| Some((parent, parent.subsection(title)?)))
        else {
            continue;
        };
        let heading = subsection_heading(parent, translation);
        if kind == Kind::Paragraphs {
            findings.extend(plain_paragraphs(file, &heading, translation));
        }
        if kind == Kind::Bullets {
            findings.extend(same_bullets(file, (english, translation), &heading));
        }
    }
    if kind == Kind::Bullets && title == "Outcomes" {
        findings.extend(outcome_count(file, english));
    }
    findings
}

fn plain_paragraphs(file: &NoteFile, heading: &str, section: &Section) -> Vec<Finding> {
    text_lines(section)
        .filter_map(|line| {
            forbidden_in_paragraph(&line.text).map(|forbidden| {
                Finding::new(
                    file,
                    Rule::Body,
                    At::Section {
                        heading: heading.to_owned(),
                        line: line.number,
                    },
                    format!(
                        "holds {forbidden}, but a paragraphs section holds plain paragraphs only"
                    ),
                )
            })
        })
        .collect()
}

/// A translation has the same number of bullets as the English; an empty one is not translated yet.
fn same_bullets(
    file: &NoteFile,
    (english, translation): (&Section, &Section),
    heading: &str,
) -> Option<Finding> {
    let translated = text_lines(translation).count();
    let expected = text_lines(english).count();
    (translated > 0 && translated != expected).then(|| {
        Finding::new(
            file,
            Rule::Body,
            At::Section {
                heading: heading.to_owned(),
                line: translation.line,
            },
            format!(
                "has {}, but the English section has {expected}",
                bullet_count(translated)
            ),
        )
    })
}

fn outcome_count(file: &NoteFile, outcomes: &Section) -> Option<Finding> {
    let count = text_lines(outcomes).count();
    (!OUTCOMES.contains(&count)).then(|| {
        Finding::new(
            file,
            Rule::Body,
            At::Section {
                heading: section_heading(outcomes),
                line: outcomes.line,
            },
            format!(
                "has {}, but a Monthly Log has {} to {}",
                bullet_count(count),
                OUTCOMES.start(),
                OUTCOMES.end()
            ),
        )
    })
}

fn bullet_count(count: usize) -> String {
    let plural = if count == 1 { "" } else { "s" };
    format!("{count} bullet{plural}")
}

fn text_lines(section: &Section) -> impl Iterator<Item = &Line> {
    section.lines.iter().filter(|line| !line.is_blank())
}

/// What a line of a paragraphs section holds that it must not (wikilinks are refused in every section).
fn forbidden_in_paragraph(line: &str) -> Option<&'static str> {
    let trimmed = line.trim();
    if is_heading(trimmed) {
        Some("a heading")
    } else if is_table_row(trimmed) {
        Some("a table")
    } else if trimmed.contains("![") {
        Some("an image")
    } else if contains_html(trimmed) {
        Some("HTML")
    } else {
        None
    }
}

/// One to six `#`, then a space. `##` and `###` headings are sections, which the vault reader has already told apart.
fn is_heading(line: &str) -> bool {
    let hashes = line.chars().take_while(|c| *c == '#').count();
    (1..=6).contains(&hashes) && line.trim_start_matches('#').starts_with(' ')
}

fn is_table_row(line: &str) -> bool {
    line.starts_with('|')
        || (line.contains('|') && line.chars().all(|c| matches!(c, '-' | '|' | ':' | ' ')))
}

/// A `<` that opens a tag, closes a tag or starts a comment, and a `>` after it.
fn contains_html(line: &str) -> bool {
    let characters: Vec<char> = line.chars().collect();
    characters.iter().enumerate().any(|(index, character)| {
        *character == '<'
            && characters
                .get(index + 1)
                .is_some_and(|next| next.is_ascii_alphabetic() || matches!(next, '/' | '!'))
            && characters.iter().skip(index + 2).any(|later| *later == '>')
    })
}

#[cfg(test)]
mod tests {
    use super::{contains_html, forbidden_in_paragraph, is_heading, is_table_row};
    use crate::test_vault::TestVault;

    const SHOP: &str = "Projects/Acme - Shop.md";
    const ENGAGEMENT: &str = "Engagements/2021-04 Acme.md";
    const LOG: &str = "Monthly Logs/2025/2025-06 Globex Log.md";

    #[test]
    fn a_wikilink_in_a_paragraph_is_reported_on_its_line() {
        let vault = TestVault::new("body_wikilink_paragraph");
        vault.replace(ENGAGEMENT, "Built backends", "Built [[Acme]] backends");

        assert_eq!(
            vault.lines(),
            [
                "Engagements/2021-04 Acme.md:23: error [body] section `## Summary` contains a wikilink, which a note body must not"
            ]
        );
    }

    #[test]
    fn a_wikilink_in_a_bullet_a_focus_or_a_translation_is_reported() {
        let vault = TestVault::new("body_wikilink_elsewhere");
        vault
            .replace(SHOP, "- Designed the order", "- Designed the [[Go]] order")
            .replace(LOG, "Search ranking", "Search [[Go]] ranking")
            .replace(SHOP, "サービスはDocker", "サービスは[[Docker]]");

        assert_eq!(
            vault.lines(),
            [
                "Monthly Logs/2025/2025-06 Globex Log.md:20: error [body] section `## Focus` contains a wikilink, which a note body must not",
                "Projects/Acme - Shop.md:62: error [body] section `## Achievements` contains a wikilink, which a note body must not",
                "Projects/Acme - Shop.md:71: error [body] section `## 日本語 > ### Description` contains a wikilink, which a note body must not",
            ]
        );
    }

    #[test]
    fn a_paragraph_holds_no_heading_table_image_or_html() {
        let vault = TestVault::new("body_paragraph_content");
        vault.replace(
            ENGAGEMENT,
            "Built backends for online shops and an accounting platform in the development team of Acme.",
            "#### Details\n| a | b |\n![chart](chart.png)\n<br>",
        );

        assert_eq!(
            vault.lines(),
            [
                "Engagements/2021-04 Acme.md:23: error [body] section `## Summary` holds a heading, but a paragraphs section holds plain paragraphs only",
                "Engagements/2021-04 Acme.md:24: error [body] section `## Summary` holds a table, but a paragraphs section holds plain paragraphs only",
                "Engagements/2021-04 Acme.md:25: error [body] section `## Summary` holds an image, but a paragraphs section holds plain paragraphs only",
                "Engagements/2021-04 Acme.md:26: error [body] section `## Summary` holds HTML, but a paragraphs section holds plain paragraphs only",
            ]
        );
    }

    #[test]
    fn a_translated_paragraph_is_held_to_the_same_rules() {
        let vault = TestVault::new("body_translated_paragraph");
        vault.replace(
            SHOP,
            "サービスはDockerコンテナで稼働する。",
            "<div>サービス</div>",
        );

        assert_eq!(
            vault.lines(),
            [
                "Projects/Acme - Shop.md:71: error [body] section `## 日本語 > ### Description` holds HTML, but a paragraphs section holds plain paragraphs only"
            ]
        );
    }

    #[test]
    fn a_translation_has_as_many_bullets_as_the_english() {
        let vault = TestVault::new("body_bullet_count");
        vault.replace(SHOP, "- カタログページの表示時間を半分に短縮した\n", "");

        assert_eq!(
            vault.lines(),
            [
                "Projects/Acme - Shop.md:73: error [body] section `## 日本語 > ### Achievements` has 1 bullet, but the English section has 2"
            ]
        );
    }

    #[test]
    fn an_untranslated_section_has_no_bullets_to_count() {
        let vault = TestVault::new("body_untranslated");
        vault.replace(SHOP, "- Entwarf den Bestell- und Zahlungsablauf\n- Halbierte die Ladezeit der Katalogseiten\n", "");

        assert_eq!(vault.lines(), Vec::<String>::new());
    }

    #[test]
    fn a_monthly_log_has_three_to_five_outcomes() {
        let vault = TestVault::new("body_outcomes");
        vault.replace(LOG, "- Started the audit log of the admin area\n", "");
        vault.replace(
            "Monthly Logs/2022/2022-05 Acme Log.md",
            "- Started the returns workflow\n",
            "- Started the returns workflow\n- One\n- Two\n- Three\n",
        );

        assert_eq!(
            vault.lines(),
            [
                "Monthly Logs/2022/2022-05 Acme Log.md:16: error [body] section `## Outcomes` has 6 bullets, but a Monthly Log has 3 to 5",
                "Monthly Logs/2025/2025-06 Globex Log.md:22: error [body] section `## Outcomes` has 2 bullets, but a Monthly Log has 3 to 5",
            ]
        );
    }

    #[test]
    fn three_and_five_outcomes_are_fine() {
        let vault = TestVault::new("body_outcomes_bounds");
        vault.replace(
            "Monthly Logs/2022/2022-05 Acme Log.md",
            "- Started the returns workflow\n",
            "- Started the returns workflow\n- One\n- Two\n",
        );

        assert_eq!(vault.lines(), Vec::<String>::new());
    }

    #[test]
    fn headings_are_one_to_six_hashes_and_a_space() {
        assert!(is_heading("# Title"));
        assert!(is_heading("###### Title"));
        assert!(is_heading("#### Title"));
        assert!(!is_heading("####### Title"));
        assert!(!is_heading("#hashtag"));
        assert!(!is_heading("text # more"));
        assert!(!is_heading("Issue #4 is fixed"));
    }

    #[test]
    fn tables_start_with_a_pipe_or_are_a_divider_row() {
        assert!(is_table_row("| a | b |"));
        assert!(is_table_row("--- | ---"));
        assert!(is_table_row(":--|--:"));
        assert!(!is_table_row("a | b"));
        assert!(!is_table_row("---"));
        assert!(!is_table_row("plain text"));
    }

    #[test]
    fn html_is_a_tag_a_closing_tag_or_a_comment() {
        for line in [
            "a <b>bold</b>",
            "</div>",
            "<!-- note -->",
            "<br/>",
            "xxxx<b>",
            "x <span class=\"a\"> y",
        ] {
            assert!(contains_html(line), "{line}");
        }
        for line in [
            "1 < 2 and 3 > 2",
            "a <3 b",
            "<",
            "<>",
            "x <",
            "a <b is open",
            "plain",
        ] {
            assert!(!contains_html(line), "{line}");
        }
    }

    #[test]
    fn a_paragraph_line_is_checked_against_each_rule_in_turn() {
        assert_eq!(forbidden_in_paragraph("## x"), Some("a heading"));
        assert_eq!(forbidden_in_paragraph("  | a |"), Some("a table"));
        assert_eq!(forbidden_in_paragraph("see ![x](y)"), Some("an image"));
        assert_eq!(forbidden_in_paragraph("<p>x</p>"), Some("HTML"));
        assert_eq!(forbidden_in_paragraph("Built the #4 feature"), None);
    }
}
