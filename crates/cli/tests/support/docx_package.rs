//! Reads a DOCX file back: its parts, and its body as the outline of paragraphs and tables a person sees.

use std::error::Error;
use std::io::{Cursor, Read};

use zip::ZipArchive;

pub(crate) type TestResult<T = ()> = Result<T, Box<dyn Error>>;

pub(crate) struct DocxPackage {
    archive: ZipArchive<Cursor<Vec<u8>>>,
}

impl DocxPackage {
    pub(crate) fn open(bytes: &[u8]) -> TestResult<Self> {
        let archive = ZipArchive::new(Cursor::new(bytes.to_vec()))?;
        Ok(Self { archive })
    }

    pub(crate) fn part(&mut self, name: &str) -> TestResult<String> {
        let mut text = String::new();
        self.archive.by_name(name)?.read_to_string(&mut text)?;
        Ok(text)
    }

    pub(crate) fn part_names(&self) -> TestResult<Vec<String>> {
        self.archive
            .file_names()
            .map(|name| Ok(name?.into_owned()))
            .collect()
    }

    pub(crate) fn modification_times(&mut self) -> TestResult<Vec<String>> {
        (0..self.archive.len())
            .map(|index| {
                let file = self.archive.by_index(index)?;
                Ok(format!("{}: {:?}", file.name()?, file.last_modified()))
            })
            .collect()
    }

    pub(crate) fn outline(&mut self) -> TestResult<String> {
        outline_of(&self.part("word/document.xml")?)
    }
}

/// One line per paragraph, indented by how deep in tables it sits, with `[table]` and `[cell]` lines for the
/// tables themselves and `•` for a paragraph in a list.
fn outline_of(document: &str) -> TestResult<String> {
    let mut outline = Outline::default();
    for tag in document.split('<').skip(1) {
        let (markup, text) = tag.split_once('>').ok_or("an unclosed tag")?;
        outline.read(markup, text);
    }
    Ok(outline.lines.join("\n"))
}

#[derive(Default)]
struct Outline {
    lines: Vec<String>,
    depth: usize,
    paragraph: Option<Paragraph>,
}

impl Outline {
    fn read(&mut self, markup: &str, text: &str) {
        let closing = markup.starts_with('/');
        let name = markup
            .trim_start_matches('/')
            .split([' ', '/'])
            .next()
            .unwrap_or_default();
        match (name, closing) {
            ("w:tbl" | "w:tc", _) => self.read_table_part(name, closing),
            ("w:p", _) => self.read_paragraph(markup, closing),
            ("w:numPr", false) => self.in_paragraph(|paragraph| paragraph.listed = true),
            // A tab stop's definition has a `w:val`; the tab character of a run has none.
            ("w:tab", false) if !markup.contains("w:val") => {
                self.in_paragraph(|paragraph| paragraph.text.push('\t'));
            }
            ("w:t", false) => {
                self.in_paragraph(|paragraph| paragraph.text.push_str(&unescaped(text)));
            }
            _ => {}
        }
    }

    fn read_table_part(&mut self, name: &str, closing: bool) {
        match (name, closing) {
            ("w:tbl", false) => {
                self.lines.push(format!("{}[table]", indent(self.depth)));
                self.depth += 1;
            }
            ("w:tbl", true) => self.depth -= 1,
            ("w:tc", false) => self.lines.push(format!("{}[cell]", indent(self.depth))),
            _ => {}
        }
    }

    fn read_paragraph(&mut self, markup: &str, closing: bool) {
        if closing {
            if let Some(finished) = self.paragraph.take() {
                self.lines
                    .push(format!("{}{}", indent(self.depth), finished.line()));
            }
        } else if !markup.ends_with('/') {
            self.paragraph = Some(Paragraph::default());
        }
    }

    fn in_paragraph(&mut self, change: impl FnOnce(&mut Paragraph)) {
        if let Some(paragraph) = self.paragraph.as_mut() {
            change(paragraph);
        }
    }
}

#[derive(Default)]
struct Paragraph {
    text: String,
    listed: bool,
}

impl Paragraph {
    fn line(&self) -> String {
        let bullet = if self.listed { "• " } else { "" };
        format!("{bullet}{}", self.text)
    }
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

fn unescaped(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}
