/// A note's body as written: its `##` sections, each with the `###` subsections under it (schema §1.6). Reading
/// it never fails; whether it has the sections its type asks for is checked when the note is read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Body {
    /// Lines before the first `##` heading.
    pub preamble: Vec<Line>,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// The heading without its `##` or `###`.
    pub title: String,
    /// The heading's line.
    pub line: usize,
    /// The lines between the heading and the next heading.
    pub lines: Vec<Line>,
    /// Empty for a `###` subsection.
    pub subsections: Vec<Self>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// 1-based, counted from the first line of the file.
    pub number: usize,
    pub text: String,
}

impl Body {
    /// `first_line` is the file line the body starts on.
    pub(crate) fn parse(text: &str, first_line: usize) -> Self {
        let mut body = Self::default();
        for (number, line) in (first_line..).zip(text.lines()) {
            body.add(number, line);
        }
        body
    }

    #[must_use]
    pub fn section(&self, title: &str) -> Option<&Section> {
        self.sections.iter().find(|section| section.title == title)
    }

    fn add(&mut self, number: usize, text: &str) {
        if let Some(title) = text.strip_prefix("## ") {
            self.sections.push(Section::new(title, number));
        } else if let Some(title) = text.strip_prefix("### ")
            && let Some(section) = self.sections.last_mut()
        {
            section.subsections.push(Section::new(title, number));
        } else {
            self.last_lines().push(Line {
                number,
                text: text.to_owned(),
            });
        }
    }

    fn last_lines(&mut self) -> &mut Vec<Line> {
        match self.sections.last_mut() {
            Some(section) => match section.subsections.last_mut() {
                Some(subsection) => &mut subsection.lines,
                None => &mut section.lines,
            },
            None => &mut self.preamble,
        }
    }
}

impl Section {
    fn new(title: &str, line: usize) -> Self {
        Self {
            title: title.trim().to_owned(),
            line,
            lines: Vec::new(),
            subsections: Vec::new(),
        }
    }

    #[must_use]
    pub fn subsection(&self, title: &str) -> Option<&Self> {
        self.subsections
            .iter()
            .find(|subsection| subsection.title == title)
    }

    #[must_use]
    pub fn is_blank(&self) -> bool {
        is_blank(&self.lines)
    }
}

impl Line {
    #[must_use]
    pub fn is_blank(&self) -> bool {
        self.text.trim().is_empty()
    }
}

pub(crate) fn is_blank(lines: &[Line]) -> bool {
    lines.iter().all(Line::is_blank)
}

#[cfg(test)]
mod tests {
    use super::{Body, Line, Section};

    fn line(number: usize, text: &str) -> Line {
        Line {
            number,
            text: text.to_owned(),
        }
    }

    #[test]
    fn headings_split_the_body_into_sections_and_subsections() {
        let body = Body::parse(
            "\n## Summary\nBuilt it.\n\n## 日本語\n\n### Summary\n構築した。\n",
            10,
        );

        assert_eq!(body.preamble, vec![line(10, "")]);
        assert_eq!(
            body.sections,
            vec![
                Section {
                    title: "Summary".to_owned(),
                    line: 11,
                    lines: vec![line(12, "Built it."), line(13, "")],
                    subsections: Vec::new(),
                },
                Section {
                    title: "日本語".to_owned(),
                    line: 14,
                    lines: vec![line(15, "")],
                    subsections: vec![Section {
                        title: "Summary".to_owned(),
                        line: 16,
                        lines: vec![line(17, "構築した。")],
                        subsections: Vec::new(),
                    }],
                },
            ]
        );
    }

    #[test]
    fn a_subsection_heading_before_any_section_is_text_before_the_sections() {
        let body = Body::parse("### Stray\n#### Deeper\n", 1);

        assert_eq!(
            body.preamble,
            vec![line(1, "### Stray"), line(2, "#### Deeper")]
        );
        assert_eq!(body.sections, Vec::new());
    }

    #[test]
    fn sections_and_subsections_are_found_by_title() {
        let body = Body::parse("## A\n## B\n### C\nx\n", 1);

        let found = body
            .section("B")
            .and_then(|section| section.subsection("C"));

        assert_eq!(found.map(|subsection| subsection.line), Some(3));
        assert!(body.section("C").is_none());
        assert!(!found.unwrap().is_blank());
        assert!(body.section("A").unwrap().is_blank());
    }
}
