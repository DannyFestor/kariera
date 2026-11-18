use rust_xlsxwriter::{Worksheet, XlsxError};

use crate::grid::{COLUMN_COUNT, COLUMN_WIDTH};

const MARGIN_SIDE_INCHES: f64 = 0.4;
const MARGIN_TOP_INCHES: f64 = 0.5;
const MARGIN_BOTTOM_INCHES: f64 = 0.6;
const MARGIN_HEADER_FOOTER_INCHES: f64 = 0.3;
/// The code of A4 in the file format.
const PAPER_SIZE_A4: u8 = 9;
const FOOTER: &str = "&C&8&P / &N";

/// A4 portrait at the sheet's own scale. The grid is narrower than the printable width, so nothing is cut off, and
/// there is no "fit to one page wide": Excel ignores manual page breaks under it, and the page breaks are what
/// keep a Project block on one page.
pub(crate) fn apply(worksheet: &mut Worksheet, last_row: u32) -> Result<(), XlsxError> {
    worksheet.set_paper_size(PAPER_SIZE_A4);
    worksheet.set_portrait();
    worksheet.set_margins(
        MARGIN_SIDE_INCHES,
        MARGIN_SIDE_INCHES,
        MARGIN_TOP_INCHES,
        MARGIN_BOTTOM_INCHES,
        MARGIN_HEADER_FOOTER_INCHES,
        MARGIN_HEADER_FOOTER_INCHES,
    );
    worksheet.set_print_center_horizontally(true);
    worksheet.set_footer(FOOTER);
    worksheet.set_print_area(0, 0, last_row, COLUMN_COUNT - 1)?;
    worksheet.set_column_range_width(0, COLUMN_COUNT - 1, COLUMN_WIDTH)?;
    Ok(())
}
