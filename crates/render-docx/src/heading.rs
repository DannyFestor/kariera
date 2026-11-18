//! Section headings in Word's Heading 1 style, so the navigation pane and the agency's own tools find them.

use docx_rs::{
    BorderType, Docx, Paragraph, ParagraphBorder, ParagraphBorderPosition, Style, StyleType,
};

use crate::blocks::{spacing, text};

const HEADING_1: &str = "Heading1";

#[derive(Debug, Clone, Copy)]
pub(crate) struct HeadingLook {
    pub(crate) size: usize,
    pub(crate) color: &'static str,
    /// The colour of a rule under the heading, if it has one.
    pub(crate) rule: Option<&'static str>,
}

pub(crate) fn with_heading_style(docx: Docx, look: HeadingLook) -> Docx {
    docx.add_style(
        Style::new(HEADING_1, StyleType::Paragraph)
            .name("heading 1")
            .based_on("Normal")
            .next("Normal")
            .size(look.size)
            .color(look.color)
            .bold()
            .outline_lvl(0),
    )
}

pub(crate) fn heading(title: &str, look: HeadingLook) -> Paragraph {
    let paragraph = Paragraph::new()
        .style(HEADING_1)
        .add_run(text(title))
        .keep_next(true)
        .line_spacing(spacing(240, 100));
    match look.rule {
        Some(color) => ruled(paragraph, color),
        None => paragraph,
    }
}

fn ruled(mut paragraph: Paragraph, color: &str) -> Paragraph {
    paragraph.property = paragraph.property.set_border(
        ParagraphBorder::new(ParagraphBorderPosition::Bottom)
            .val(BorderType::Single)
            .size(8)
            .color(color)
            .space(2),
    );
    paragraph
}
