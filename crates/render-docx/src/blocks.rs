//! What both layouts build their pages from: paragraphs and tables that stay paragraphs and tables in Word.

use docx_rs::{
    AlignmentType, Docx, LineSpacing, Paragraph, Run, Shading, Tab, TabValueType, Table, TableCell,
    TableCellBorder, TableCellBorderPosition, TableCellBorders, TableCellMargins, TableLayoutType,
    TableRow, WidthType,
};

use crate::theme::RULE;

pub(crate) enum Block {
    Paragraph(Box<Paragraph>),
    Table(Box<Table>),
}

impl From<Paragraph> for Block {
    fn from(paragraph: Paragraph) -> Self {
        Self::Paragraph(Box::new(paragraph))
    }
}

impl From<Table> for Block {
    fn from(table: Table) -> Self {
        Self::Table(Box::new(table))
    }
}

pub(crate) fn add_blocks(docx: Docx, blocks: Vec<Block>) -> Docx {
    blocks.into_iter().fold(docx, |docx, block| match block {
        Block::Paragraph(paragraph) => docx.add_paragraph(*paragraph),
        Block::Table(table) => docx.add_table(*table),
    })
}

pub(crate) fn text(content: &str) -> Run {
    Run::new().add_text(content)
}

pub(crate) fn tab() -> Run {
    Run::new().add_tab()
}

pub(crate) fn spacing(before: u32, after: u32) -> LineSpacing {
    LineSpacing::new().before(before).after(after)
}

pub(crate) fn right_tab(position: usize) -> Tab {
    Tab::new().val(TabValueType::Right).pos(position)
}

pub(crate) fn body_paragraphs(texts: &[String]) -> Vec<Paragraph> {
    texts
        .iter()
        .map(|content| {
            Paragraph::new()
                .add_run(text(content))
                .align(AlignmentType::Both)
                .line_spacing(spacing(0, 100))
        })
        .collect()
}

/// Keeps the paragraphs on one page, as far as they fit on one.
pub(crate) fn keep_together(paragraphs: Vec<Paragraph>) -> Vec<Paragraph> {
    let last = paragraphs.len().saturating_sub(1);
    paragraphs
        .into_iter()
        .enumerate()
        .map(|(index, paragraph)| paragraph.keep_next(index < last))
        .collect()
}

/// Word merges two tables with nothing between them into one, so tables in a row are told apart by a thin
/// paragraph.
pub(crate) fn spacer(height: usize) -> Paragraph {
    Paragraph::new().size(height).line_spacing(spacing(0, 0))
}

pub(crate) fn no_borders() -> TableCellBorders {
    TableCellBorders::with_empty()
}

pub(crate) fn bottom_rule() -> TableCellBorders {
    no_borders().set(
        TableCellBorder::new(TableCellBorderPosition::Bottom)
            .size(4)
            .color(RULE),
    )
}

/// A cell has to end in a paragraph, so one that ends in a nested table gets a thin one.
pub(crate) fn cell(width: usize, borders: TableCellBorders, blocks: Vec<Block>) -> TableCell {
    let ends_in_table = matches!(blocks.last(), Some(Block::Table(_)));
    let filled = blocks
        .into_iter()
        .fold(TableCell::new(), |cell, block| match block {
            Block::Paragraph(paragraph) => cell.add_paragraph(*paragraph),
            Block::Table(table) => cell.add_table(*table),
        });
    let ended = if ends_in_table {
        filled.add_paragraph(spacer(8))
    } else {
        filled
    };
    ended.width(width, WidthType::Dxa).set_borders(borders)
}

pub(crate) fn row(cells: Vec<TableCell>) -> TableRow {
    TableRow::new(cells)
}

pub(crate) fn shaded(mut paragraph: Paragraph, fill: &str) -> Paragraph {
    paragraph.property = paragraph.property.shading(Shading::new().fill(fill));
    paragraph
}

pub(crate) fn plain_cell(width: usize, blocks: Vec<Block>) -> TableCell {
    cell(width, no_borders(), blocks)
}

pub(crate) fn paragraph_cell(width: usize, paragraphs: Vec<Paragraph>) -> TableCell {
    plain_cell(width, paragraphs.into_iter().map(Block::from).collect())
}

/// Fixed column widths and no borders of its own: the cells carry any rules.
pub(crate) fn table(widths: &[usize], rows: Vec<TableRow>, margins: TableCellMargins) -> Table {
    Table::new(rows)
        .set_grid(widths.to_vec())
        .layout(TableLayoutType::Fixed)
        .width(widths.iter().sum(), WidthType::Dxa)
        .margins(margins)
        .clear_all_border()
}
