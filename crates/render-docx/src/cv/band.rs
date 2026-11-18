//! The header band: a one-row table with the name and headline, the contact lines and, on the German CV, the photo.

use docx_rs::{
    AlignmentType, Paragraph, Pic, Run, Shading, Table, TableCell, TableCellMargins, VAlignType,
};
use kariera_documents::{Contact, Cv};

use super::{BAND, HEADLINE_ON_BAND};
use crate::blocks::{Block, cell, no_borders, row, spacing, table, text};
use crate::link::hyperlink;
use crate::theme::TEXT_WIDTH;

const NAME_SIZE: usize = 56;
const HEADLINE_SIZE: usize = 24;
const CONTACT_SIZE: usize = 16;
const WHITE: &str = "FFFFFF";
const CONTACT_WIDTH: usize = 3400;
const PHOTO_WIDTH: usize = 1700;

pub(super) fn band(cv: &Cv, photo: Option<Pic>) -> Table {
    let photo_width = photo.as_ref().map_or(0, |_| PHOTO_WIDTH);
    let name_width = TEXT_WIDTH - CONTACT_WIDTH - photo_width;
    let mut widths = vec![name_width, CONTACT_WIDTH];
    let mut cells = vec![
        banded(name_width, vec![name_line(cv), headline_line(cv)]),
        banded(CONTACT_WIDTH, contact_lines(&cv.contact)),
    ];
    if let Some(photo) = photo {
        widths.push(PHOTO_WIDTH);
        cells.push(banded(PHOTO_WIDTH, vec![photo_line(photo)]));
    }
    table(
        &widths,
        vec![row(cells)],
        TableCellMargins::new().margin(200, 240, 160, 240),
    )
}

fn banded(width: usize, paragraphs: Vec<Paragraph>) -> TableCell {
    cell(
        width,
        no_borders(),
        paragraphs.into_iter().map(Block::from).collect(),
    )
    .shading(Shading::new().fill(BAND))
    .vertical_align(VAlignType::Bottom)
}

fn name_line(cv: &Cv) -> Paragraph {
    Paragraph::new()
        .add_run(text(&cv.name).size(NAME_SIZE).bold().color(WHITE))
        .line_spacing(spacing(0, 0))
}

fn headline_line(cv: &Cv) -> Paragraph {
    Paragraph::new().add_run(
        text(&cv.headline)
            .size(HEADLINE_SIZE)
            .color(HEADLINE_ON_BAND),
    )
}

fn photo_line(photo: Pic) -> Paragraph {
    Paragraph::new()
        .add_run(Run::new().add_image(photo))
        .align(AlignmentType::Right)
}

fn contact_lines(contact: &Contact) -> Vec<Paragraph> {
    let plain = [&contact.location, &contact.phone]
        .into_iter()
        .flatten()
        .map(|line| contact_line(vec![text(line).size(CONTACT_SIZE).color(WHITE)]));
    let email = contact
        .email
        .iter()
        .map(|address| contact_line(link(&format!("mailto:{address}"), address)));
    let urls = [&contact.portfolio_url, &contact.github_url]
        .into_iter()
        .flatten()
        .map(|url| contact_line(link(url, url.trim_start_matches("https://"))));
    plain.chain(email).chain(urls).collect()
}

fn link(target: &str, shown: &str) -> Vec<Run> {
    hyperlink(target, shown, WHITE)
        .into_iter()
        .map(|run| run.size(CONTACT_SIZE))
        .collect()
}

fn contact_line(runs: Vec<Run>) -> Paragraph {
    runs.into_iter()
        .fold(Paragraph::new(), Paragraph::add_run)
        .align(AlignmentType::Right)
        .line_spacing(spacing(0, 0))
}
