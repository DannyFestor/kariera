use kariera_domain::{Bullets, Paragraphs, Text, Translated};

use crate::body::{Body, Line, Section};
use crate::problems::{Problems, Reported};
use crate::{Location, Problem};

const JAPANESE: &str = "日本語";
const GERMAN: &str = "Deutsch";

/// Reads a section's content. The `&str` is the section's heading, for the problem's location.
type ReadContent<T> = fn(&Section, &str) -> Result<T, Problem>;

/// Reads the body sections a note type asks for, in the order it asks for them (schema §1.6), and at the end
/// checks that the body has exactly those headings.
#[derive(Debug)]
pub(crate) struct Sections<'a> {
    body: &'a Body,
    problems: &'a Problems,
    titles: Vec<&'static str>,
    translated: bool,
}

impl<'a> Sections<'a> {
    pub(crate) const fn new(body: &'a Body, problems: &'a Problems) -> Self {
        Self {
            body,
            problems,
            titles: Vec::new(),
            translated: false,
        }
    }

    pub(crate) fn paragraphs(
        &mut self,
        title: &'static str,
    ) -> Result<Translated<Paragraphs>, Reported> {
        self.translated(title, |section, _| Ok(paragraphs(section)))
    }

    pub(crate) fn bullets(&mut self, title: &'static str) -> Result<Translated<Bullets>, Reported> {
        self.translated(title, bullets)
    }

    pub(crate) fn english_bullets(&mut self, title: &'static str) -> Result<Bullets, Reported> {
        self.english(title, bullets)
    }

    pub(crate) fn one_line(&mut self, title: &'static str) -> Result<Text, Reported> {
        self.english(title, one_line)
    }

    pub(crate) fn finish(self) {
        if self.titles.is_empty() {
            self.check_empty();
        } else {
            self.check_preamble();
            self.check_headings();
            self.check_english_has_no_subsections();
            self.check_translations();
        }
    }

    /// A missing section is not reported here but by `finish`, which sees every heading.
    fn english<T>(&mut self, title: &'static str, read: ReadContent<T>) -> Result<T, Reported> {
        self.titles.push(title);
        let section = self.body.section(title).ok_or(Reported)?;
        self.read(section, &format!("## {title}"), read)
    }

    fn translated<T>(
        &mut self,
        title: &'static str,
        read: ReadContent<T>,
    ) -> Result<Translated<T>, Reported> {
        self.translated = true;
        let en = self.english(title, read);
        let ja = self.translation(JAPANESE, title, read);
        let de = self.translation(GERMAN, title, read);
        Ok(Translated {
            en: en?,
            ja: ja?,
            de: de?,
        })
    }

    /// An empty subsection is not translated yet.
    fn translation<T>(
        &self,
        language: &str,
        title: &str,
        read: ReadContent<T>,
    ) -> Result<Option<T>, Reported> {
        let subsection = self
            .body
            .section(language)
            .and_then(|section| section.subsection(title))
            .ok_or(Reported)?;
        if subsection.is_blank() {
            return Ok(None);
        }
        self.read(subsection, &format!("## {language} > ### {title}"), read)
            .map(Some)
    }

    fn read<T>(
        &self,
        section: &Section,
        heading: &str,
        read: ReadContent<T>,
    ) -> Result<T, Reported> {
        read(section, heading).map_err(|problem| {
            self.problems.record(problem);
            Reported
        })
    }

    fn check_empty(&self) {
        let first_text = self
            .body
            .preamble
            .iter()
            .find(|line| !line.is_blank())
            .map(|line| line.number);
        let first_heading = self.body.sections.first().map(|section| section.line);
        if let Some(line) = first_text.or(first_heading) {
            self.record(Location::Note, line, "must have an empty body");
        }
    }

    fn check_preamble(&self) {
        if let Some(line) = self.body.preamble.iter().find(|line| !line.is_blank()) {
            self.record(
                Location::Note,
                line.number,
                "must have no text before its first `##` heading",
            );
        }
    }

    fn check_headings(&self) {
        let mut expected = self.titles.clone();
        if self.translated {
            expected.extend([JAPANESE, GERMAN]);
        }
        let actual = titles(&self.body.sections);
        if actual != expected {
            self.problems.record(Problem::new(
                Location::Note,
                None,
                format!(
                    "must have the sections {} in this order, not {}",
                    headings("##", &expected),
                    headings("##", &actual)
                ),
            ));
        }
    }

    fn check_english_has_no_subsections(&self) {
        for title in &self.titles {
            let first_subsection = self
                .body
                .section(title)
                .and_then(|section| section.subsections.first());
            if let Some(subsection) = first_subsection {
                let location = Location::Section(format!("## {title}"));
                self.record(location, subsection.line, "must not have `###` subsections");
            }
        }
    }

    fn check_translations(&self) {
        if !self.translated {
            return;
        }
        for language in [JAPANESE, GERMAN] {
            if let Some(section) = self.body.section(language) {
                self.check_translation(section);
            }
        }
    }

    fn check_translation(&self, section: &Section) {
        let location = || Location::Section(format!("## {}", section.title));
        if let Some(line) = section.lines.iter().find(|line| !line.is_blank()) {
            self.record(
                location(),
                line.number,
                "must hold its text in `###` subsections",
            );
        }
        let actual = titles(&section.subsections);
        if actual != self.titles {
            let message = format!(
                "must have the subsections {} in this order, not {}",
                headings("###", &self.titles),
                headings("###", &actual)
            );
            self.record(location(), section.line, &message);
        }
    }

    fn record(&self, location: Location, line: usize, message: &str) {
        self.problems
            .record(Problem::new(location, Some(line), message));
    }
}

fn titles(sections: &[Section]) -> Vec<&str> {
    sections
        .iter()
        .map(|section| section.title.as_str())
        .collect()
}

fn headings(marker: &str, titles: &[&str]) -> String {
    if titles.is_empty() {
        return "none".to_owned();
    }
    titles
        .iter()
        .map(|title| format!("`{marker} {title}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Without the blank lines around the paragraphs.
fn paragraphs(section: &Section) -> Paragraphs {
    let mut lines: Vec<&Line> = section
        .lines
        .iter()
        .skip_while(|line| line.is_blank())
        .collect();
    while lines.last().is_some_and(|line| line.is_blank()) {
        lines.pop();
    }
    let texts: Vec<&str> = lines.iter().map(|line| line.text.as_str()).collect();
    Paragraphs(texts.join("\n"))
}

/// Blank lines between bullets are allowed; anything else must be a bullet.
fn bullets(section: &Section, heading: &str) -> Result<Bullets, Problem> {
    text_lines(section)
        .map(|line| bullet(line, heading))
        .collect()
}

fn bullet(line: &Line, heading: &str) -> Result<Text, Problem> {
    let item = line
        .text
        .strip_prefix("- ")
        .ok_or_else(|| problem(heading, line.number, "must hold one `- ` bullet per line"))?;
    item.parse()
        .map_err(|error| problem(heading, line.number, &format!("has a bullet that {error}")))
}

fn one_line(section: &Section, heading: &str) -> Result<Text, Problem> {
    let mut lines = text_lines(section);
    match (lines.next(), lines.next()) {
        (Some(line), None) => line
            .text
            .parse()
            .map_err(|error| problem(heading, line.number, &format!("has a line that {error}"))),
        (None, _) => Err(problem(heading, section.line, "must hold exactly one line")),
        (Some(_), Some(second)) => Err(problem(
            heading,
            second.number,
            "must hold exactly one line",
        )),
    }
}

fn text_lines(section: &Section) -> impl Iterator<Item = &Line> {
    section.lines.iter().filter(|line| !line.is_blank())
}

fn problem(heading: &str, line: usize, message: &str) -> Problem {
    Problem::new(Location::Section(heading.to_owned()), Some(line), message)
}

#[cfg(test)]
mod tests {
    use kariera_domain::{Bullets, Paragraphs, Text, Translated};

    use super::Sections;
    use crate::body::Body;
    use crate::problems::{Problems, Reported};
    use crate::{Location, Problem};

    fn read<T>(
        body: &str,
        read: impl FnOnce(&mut Sections<'_>) -> Result<T, Reported>,
    ) -> Result<T, Vec<Problem>> {
        let body = Body::parse(body, 1);
        let problems = Problems::default();
        let mut sections = Sections::new(&body, &problems);
        let value = read(&mut sections);
        sections.finish();
        problems.into_result(value)
    }

    fn section_problem(heading: &str, line: usize, message: &str) -> Problem {
        Problem::new(Location::Section(heading.to_owned()), Some(line), message)
    }

    fn note_problem(line: Option<usize>, message: &str) -> Problem {
        Problem::new(Location::Note, line, message)
    }

    fn text(value: &str) -> Text {
        value.parse().unwrap()
    }

    const PROJECT: &str = "\n## Description\n\nBuilt the shop.\n\nIn Go.\n\n## Achievements\n\n- Shipped\n- Scaled\n\n\
                           ## 日本語\n\n### Description\n\nショップを構築した。\n\n### Achievements\n\n\
                           ## Deutsch\n\n### Description\n\n### Achievements\n\n- Ausgeliefert\n- Skaliert\n";

    #[test]
    fn translated_sections_are_read_with_their_translations() {
        let prose = read(PROJECT, |sections| {
            let description = sections.paragraphs("Description");
            let achievements = sections.bullets("Achievements");
            Ok((description?, achievements?))
        });

        assert_eq!(
            prose,
            Ok((
                Translated {
                    en: Paragraphs("Built the shop.\n\nIn Go.".to_owned()),
                    ja: Some(Paragraphs("ショップを構築した。".to_owned())),
                    de: None,
                },
                Translated {
                    en: vec![text("Shipped"), text("Scaled")],
                    ja: None,
                    de: Some(vec![text("Ausgeliefert"), text("Skaliert")]),
                },
            ))
        );
    }

    #[test]
    fn english_only_sections_have_no_translations() {
        let log = read(
            "## Focus\n\nSearch.\n\n## Outcomes\n\n- Shipped search\n",
            |sections| {
                let focus = sections.one_line("Focus");
                let outcomes = sections.english_bullets("Outcomes");
                Ok((focus?, outcomes?))
            },
        );

        assert_eq!(log, Ok((text("Search."), vec![text("Shipped search")])));
    }

    #[test]
    fn a_type_without_sections_has_an_empty_body() {
        assert_eq!(read("\n  \n", |_| Ok(())), Ok(()));
        assert_eq!(
            read("\nSome text\n", |_| Ok(())),
            Err(vec![note_problem(Some(2), "must have an empty body")])
        );
        assert_eq!(
            read("## Notes\n", |_| Ok(())),
            Err(vec![note_problem(Some(1), "must have an empty body")])
        );
    }

    #[test]
    fn missing_or_extra_sections_are_reported_once_for_the_body() {
        let summary = read(
            "## Summary\n## Extra\n## 日本語\n### Summary\n",
            |sections| sections.paragraphs("Summary"),
        );

        assert_eq!(
            summary,
            Err(vec![note_problem(
                None,
                "must have the sections `## Summary`, `## 日本語`, `## Deutsch` in this order, not \
                 `## Summary`, `## Extra`, `## 日本語`"
            )])
        );
        assert_eq!(
            read("", |sections| sections.english_bullets("Outcomes")),
            Err(vec![note_problem(
                None,
                "must have the sections `## Outcomes` in this order, not none"
            )])
        );
    }

    #[test]
    fn text_before_the_first_section_is_reported() {
        let focus = read("Intro\n## Focus\nSearch.\n", |sections| {
            sections.one_line("Focus")
        });

        assert_eq!(
            focus,
            Err(vec![note_problem(
                Some(1),
                "must have no text before its first `##` heading"
            )])
        );
    }

    #[test]
    fn subsections_belong_to_translations_only() {
        let focus = read("## Focus\nSearch.\n### Detail\n", |sections| {
            sections.one_line("Focus")
        });

        assert_eq!(
            focus,
            Err(vec![section_problem(
                "## Focus",
                3,
                "must not have `###` subsections"
            )])
        );
    }

    #[test]
    fn a_translation_holds_the_english_subsections_and_nothing_else() {
        let summary = read(
            "## Summary\nDid it.\n## 日本語\nやった。\n### Overview\n## Deutsch\n### Summary\n",
            |sections| sections.paragraphs("Summary"),
        );

        assert_eq!(
            summary,
            Err(vec![
                section_problem("## 日本語", 4, "must hold its text in `###` subsections"),
                section_problem(
                    "## 日本語",
                    3,
                    "must have the subsections `### Summary` in this order, not `### Overview`"
                ),
            ])
        );
    }

    #[test]
    fn a_bullets_section_holds_only_bullets() {
        let outcomes = read("## Outcomes\n- Shipped\nAlso this\n", |sections| {
            sections.english_bullets("Outcomes")
        });

        assert_eq!(
            outcomes,
            Err(vec![section_problem(
                "## Outcomes",
                3,
                "must hold one `- ` bullet per line"
            )])
        );
    }

    #[test]
    fn a_bullet_is_single_trimmed_text() {
        let outcomes = read("## Outcomes\n- Shipped \n", |sections| {
            sections.english_bullets("Outcomes")
        });

        assert_eq!(
            outcomes,
            Err(vec![section_problem(
                "## Outcomes",
                2,
                "has a bullet that must not start or end with whitespace"
            )])
        );
    }

    #[test]
    fn a_translated_bullet_is_reported_under_its_language() {
        let achievements: Result<Translated<Bullets>, _> = read(
            "## Achievements\n- A\n## 日本語\n### Achievements\n* B\n## Deutsch\n### Achievements\n",
            |sections| sections.bullets("Achievements"),
        );

        assert_eq!(
            achievements,
            Err(vec![section_problem(
                "## 日本語 > ### Achievements",
                5,
                "must hold one `- ` bullet per line"
            )])
        );
    }

    #[test]
    fn a_one_line_section_holds_exactly_one_line() {
        let empty = read("## Focus\n\n", |sections| sections.one_line("Focus"));
        let two = read("## Focus\nOne\nTwo\n", |sections| {
            sections.one_line("Focus")
        });
        let padded = read("## Focus\n One\n", |sections| sections.one_line("Focus"));

        assert_eq!(
            empty,
            Err(vec![section_problem(
                "## Focus",
                1,
                "must hold exactly one line"
            )])
        );
        assert_eq!(
            two,
            Err(vec![section_problem(
                "## Focus",
                3,
                "must hold exactly one line"
            )])
        );
        assert_eq!(
            padded,
            Err(vec![section_problem(
                "## Focus",
                2,
                "has a line that must not start or end with whitespace"
            )])
        );
    }
}
