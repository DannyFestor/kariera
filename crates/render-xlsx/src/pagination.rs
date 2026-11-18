//! Where a page break goes so that no block is split across two pages.

use std::ops::Range;

use crate::fit::DEFAULT_ROW_HEIGHT_POINTS;

/// A4's height less the margins, with a little to spare: the row heights are estimates.
const PRINTABLE_HEIGHT_POINTS: f64 = 740.0;

/// A block that is taller than a page splits where Excel splits it; the others start a page when they would not
/// fit on the rest of the current one.
pub(crate) fn page_breaks(
    row_heights: &dyn Fn(u32) -> f64,
    row_count: u32,
    blocks: &[Range<u32>],
) -> Vec<u32> {
    let mut breaks = Vec::new();
    let mut used = 0.0;
    for row in 0..row_count {
        let block_height = blocks
            .iter()
            .find(|block| block.start == row)
            .map(|block| block.clone().map(row_heights).sum::<f64>());
        if let Some(height) = block_height
            && used > 0.0
            && used + height > PRINTABLE_HEIGHT_POINTS
        {
            breaks.push(row);
            used = 0.0;
        }
        let height = row_heights(row);
        if used + height > PRINTABLE_HEIGHT_POINTS {
            used = 0.0;
        }
        used += height;
    }
    breaks
}

pub(crate) fn height_or_default(height: Option<&f64>) -> f64 {
    height.copied().unwrap_or(DEFAULT_ROW_HEIGHT_POINTS)
}
