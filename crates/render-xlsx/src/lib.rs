//! Renders the Skill Sheet model to an XLSX that agencies edit: real cells, no images, A4 print setup.
#![expect(
    clippy::multiple_crate_versions,
    reason = "zeroize_derive, behind rust_xlsxwriter, is on syn 2 while serde_derive and thiserror-impl are on syn 3 (deny.toml lists it under `skip`)"
)]

mod canvas;
mod fit;
mod grid;
mod header;
mod look;
mod pagination;
mod print_setup;
mod profile;
mod project;
mod properties;
mod self_pr;
mod skills;
mod text;

use kariera_documents::SkillSheet;
use rust_xlsxwriter::{Workbook, XlsxError};
use thiserror::Error;

use canvas::Canvas;

#[derive(Debug, Error)]
#[error("cannot write the Skill Sheet as XLSX: {0}")]
pub struct RenderError(#[from] XlsxError);

pub fn render(sheet: &SkillSheet) -> Result<Vec<u8>, RenderError> {
    let mut workbook = Workbook::new();
    workbook.set_properties(&properties::of(sheet)?);
    write_worksheet(workbook.add_worksheet(), sheet)?;
    Ok(workbook.save_to_buffer()?)
}

fn write_worksheet(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    sheet: &SkillSheet,
) -> Result<(), XlsxError> {
    worksheet.set_name(sheet.labels.title)?;
    let mut canvas = Canvas::new(worksheet);
    header::write(&mut canvas, sheet)?;
    canvas.next_row();
    profile::write(&mut canvas, sheet)?;
    canvas.next_row();
    skills::write(&mut canvas, sheet)?;
    canvas.next_row();
    self_pr::write(&mut canvas, sheet)?;
    project::write_all(&mut canvas, sheet)?;
    let last_row = canvas.row().saturating_sub(1);
    canvas.finish()?;
    print_setup::apply(worksheet, last_row)
}
