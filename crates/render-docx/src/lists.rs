//! Bulleted lists as Word's own: a numbering definition and paragraphs that use it, so a list stays a list when edited.

use docx_rs::{
    AbstractNumbering, Docx, IndentLevel, Level, LevelJc, LevelText, NumberFormat, Numbering,
    NumberingId, Paragraph, SpecialIndentType, Start,
};

use crate::blocks::{spacing, text};

const BULLET_LIST: usize = 1;
const INDENT: i32 = 260;

pub(crate) fn with_bullets(docx: Docx, color: &str) -> Docx {
    let level = Level::new(
        0,
        Start::new(1),
        NumberFormat::new("bullet"),
        LevelText::new("•"),
        LevelJc::new("left"),
    )
    .indent(
        Some(INDENT),
        Some(SpecialIndentType::Hanging(INDENT)),
        None,
        None,
    )
    .color(color);
    docx.add_abstract_numbering(AbstractNumbering::new(BULLET_LIST).add_level(level))
        .add_numbering(Numbering::new(BULLET_LIST, BULLET_LIST))
}

pub(crate) fn bullet(content: &str) -> Paragraph {
    Paragraph::new()
        .add_run(text(content))
        .numbering(NumberingId::new(BULLET_LIST), IndentLevel::new(0))
        .line_spacing(spacing(0, 20))
}
