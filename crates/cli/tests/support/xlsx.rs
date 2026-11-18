//! Reads an XLSX the way a spreadsheet program does: from the XML inside the zip archive. Only what the renderer
//! writes is understood: text cells (shared strings) on the first worksheet.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use zip::ZipArchive;

pub(crate) struct Xlsx {
    archive: ZipArchive<Cursor<Vec<u8>>>,
    shared_strings: Vec<String>,
    pub(crate) sheet: String,
}

impl Xlsx {
    pub(crate) fn open(bytes: Vec<u8>) -> Self {
        let archive = ZipArchive::new(Cursor::new(bytes)).expect("an XLSX is a zip archive");
        let mut xlsx = Self {
            archive,
            shared_strings: Vec::new(),
            sheet: String::new(),
        };
        xlsx.shared_strings = elements(&xlsx.part("xl/sharedStrings.xml"), "si")
            .iter()
            .map(|item| unescape(&text_of(item)))
            .collect();
        xlsx.sheet = xlsx.part("xl/worksheets/sheet1.xml");
        xlsx
    }

    pub(crate) fn part(&mut self, name: &str) -> String {
        let mut text = String::new();
        self.archive
            .by_name(name)
            .unwrap_or_else(|_| panic!("the archive has {name}"))
            .read_to_string(&mut text)
            .unwrap();
        text
    }

    pub(crate) fn part_names(&self) -> Vec<String> {
        self.archive
            .file_names()
            .map(|name| name.expect("entry names are valid").into_owned())
            .collect()
    }

    /// Every text cell as `(row, column, text)`, both counted from 0.
    pub(crate) fn cells(&self) -> Vec<(u32, u32, String)> {
        elements(&self.sheet, "c")
            .iter()
            .filter(|cell| attribute(cell, "t").as_deref() == Some("s"))
            .map(|cell| {
                let (row, column) = position(&attribute(cell, "r").unwrap());
                let index: usize = text_of(cell).parse().unwrap();
                (row, column, self.shared_strings[index].clone())
            })
            .collect()
    }

    pub(crate) fn cell(&self, row: u32, column: u32) -> Option<String> {
        self.cells()
            .into_iter()
            .find(|cell| (cell.0, cell.1) == (row, column))
            .map(|cell| cell.2)
    }

    /// The row and column of the first cell holding exactly `text`.
    pub(crate) fn find(&self, text: &str) -> Option<(u32, u32)> {
        self.cells()
            .into_iter()
            .find(|cell| cell.2 == text)
            .map(|cell| (cell.0, cell.1))
    }

    /// The text of the cell `columns` to the right of the first cell holding exactly `label`.
    pub(crate) fn beside(&self, label: &str, columns: u32) -> Option<String> {
        let (row, column) = self.find(label)?;
        self.cell(row, column + columns)
    }

    /// The fill colour of the cell, as `RRGGBB`, or `None` when it has none.
    pub(crate) fn fill_of(&mut self, row: u32, column: u32) -> Option<String> {
        let format = self.format_of(row, column)?;
        let fill: usize = attribute(&format, "fillId")?.parse().ok()?;
        let fills = section(&self.part("xl/styles.xml"), "fills");
        let color = elements(&elements(&fills, "fill")[fill], "fgColor");
        attribute(color.first()?, "rgb").map(|rgb| rgb.trim_start_matches("FF").to_owned())
    }

    pub(crate) fn font_name_of(&mut self, row: u32, column: u32) -> Option<String> {
        let format = self.format_of(row, column)?;
        let font: usize = attribute(&format, "fontId")?.parse().ok()?;
        let fonts = section(&self.part("xl/styles.xml"), "fonts");
        attribute(
            elements(&elements(&fonts, "font")[font], "name").first()?,
            "val",
        )
    }

    /// The `<xf>` element that formats the cell.
    fn format_of(&mut self, row: u32, column: u32) -> Option<String> {
        let reference = reference(row, column);
        let style: usize = elements(&self.sheet, "c")
            .iter()
            .find(|cell| attribute(cell, "r").as_deref() == Some(reference.as_str()))
            .and_then(|cell| attribute(cell, "s"))?
            .parse()
            .ok()?;
        let formats = section(&self.part("xl/styles.xml"), "cellXfs");
        elements(&formats, "xf").get(style).cloned()
    }

    /// Row heights in points, by row counted from 0, for the rows that set one.
    pub(crate) fn row_heights(&self) -> BTreeMap<u32, f64> {
        elements(&self.sheet, "row")
            .iter()
            .filter_map(|row| {
                let number: u32 = attribute(row, "r")?.parse().ok()?;
                let height: f64 = attribute(row, "ht")?.parse().ok()?;
                Some((number - 1, height))
            })
            .collect()
    }

    /// Merged ranges as `(first_row, first_column, last_row, last_column)`.
    pub(crate) fn merges(&self) -> Vec<(u32, u32, u32, u32)> {
        elements(&self.sheet, "mergeCell")
            .iter()
            .map(|merge| {
                let range = attribute(merge, "ref").unwrap();
                let (first, last) = range.split_once(':').unwrap();
                let ((first_row, first_column), (last_row, last_column)) =
                    (position(first), position(last));
                (first_row, first_column, last_row, last_column)
            })
            .collect()
    }

    /// The rows a manual page break comes before, counted from 0.
    pub(crate) fn page_breaks(&self) -> Vec<u32> {
        elements(&self.sheet, "brk")
            .iter()
            .map(|page_break| attribute(page_break, "id").unwrap().parse().unwrap())
            .collect()
    }

    /// Column widths in character units, by column counted from 0.
    pub(crate) fn column_widths(&self) -> BTreeMap<u32, f64> {
        let mut widths = BTreeMap::new();
        for column in elements(&self.sheet, "col") {
            let first: u32 = attribute(&column, "min").unwrap().parse().unwrap();
            let last: u32 = attribute(&column, "max").unwrap().parse().unwrap();
            let width: f64 = attribute(&column, "width").unwrap().parse().unwrap();
            widths.extend((first..=last).map(|number| (number - 1, width)));
        }
        widths
    }
}

fn reference(row: u32, column: u32) -> String {
    let letter = char::from_u32(u32::from('A') + column).expect("the column is within A to Z");
    format!("{letter}{}", row + 1)
}

/// The XML between the first `<name` and the closing `</name>`.
fn section(xml: &str, name: &str) -> String {
    let (_, after) = xml.split_once(&format!("<{name}")).unwrap();
    after
        .split_once(&format!("</{name}>"))
        .unwrap()
        .0
        .to_owned()
}

fn position(reference: &str) -> (u32, u32) {
    let (letters, digits) =
        reference.split_at(reference.find(|c: char| c.is_ascii_digit()).unwrap());
    let column = letters.chars().fold(0, |sum, letter| {
        sum * 26 + (u32::from(letter) - u32::from('A') + 1)
    });
    (digits.parse::<u32>().unwrap() - 1, column - 1)
}

/// Every `<name …>…</name>` and `<name …/>` in the XML, as its full text. Elements of one name do not nest.
fn elements(xml: &str, name: &str) -> Vec<String> {
    xml.split(&format!("<{name}"))
        .skip(1)
        .filter(|chunk| chunk.starts_with([' ', '>', '/']))
        .map(|chunk| {
            let (tag, rest) = chunk.split_at(chunk.find('>').unwrap() + 1);
            if tag.ends_with("/>") {
                format!("<{name}{tag}")
            } else {
                let body = rest.split(&format!("</{name}>")).next().unwrap();
                format!("<{name}{tag}{body}</{name}>")
            }
        })
        .collect()
}

fn attribute(element: &str, name: &str) -> Option<String> {
    let tag = element.split('>').next()?;
    let value = tag
        .split(&format!(" {name}=\""))
        .nth(1)?
        .split('"')
        .next()?;
    Some(unescape(value))
}

/// The text between the tags of every `<v>` or `<t>` child, joined.
fn text_of(element: &str) -> String {
    ["v", "t"]
        .into_iter()
        .flat_map(|tag| elements(element, tag))
        .map(|child| {
            child
                .split_once('>')
                .unwrap()
                .1
                .rsplit_once("</")
                .unwrap()
                .0
                .to_owned()
        })
        .collect()
}

fn unescape(text: &str) -> String {
    text.replace("&#xA;", "\n")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
}
