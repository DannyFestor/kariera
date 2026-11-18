//! The DOCX package around a layout: page, defaults, and the bytes of the file.

use std::io::Cursor;

use docx_rs::{Docx, PageMargin};

use crate::RenderError;
use crate::theme::{PAGE_HEIGHT, PAGE_WIDTH, SIDE_MARGIN, Typeface};

/// Vertical margins differ per document; the side margins are the same.
pub(crate) struct Page {
    pub(crate) top: i32,
    pub(crate) bottom: i32,
}

pub(crate) fn base(typeface: Typeface, size: usize, page: &Page) -> Docx {
    Docx::new()
        .page_size(PAGE_WIDTH, PAGE_HEIGHT)
        .page_margin(
            PageMargin::new()
                .top(page.top)
                .bottom(page.bottom)
                .left(SIDE_MARGIN)
                .right(SIDE_MARGIN)
                .header(page.top / 2)
                .footer(page.bottom / 2),
        )
        .default_fonts(typeface.run_fonts())
        .default_size(size)
}

/// `Docx::new` gives the file the dates 1970-01-01 and the same bytes each time; nothing here adds a clock.
pub(crate) fn bytes(docx: Docx) -> Result<Vec<u8>, RenderError> {
    let mut buffer = Cursor::new(Vec::new());
    docx.build()
        .pack(&mut buffer)
        .map_err(|error| RenderError::Pack(error.to_string()))?;
    Ok(buffer.into_inner())
}
