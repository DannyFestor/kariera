use std::collections::BTreeMap;
use std::ops::Range;

use rust_xlsxwriter::{Worksheet, XlsxError};

use crate::fit::{MAX_ROW_HEIGHT_POINTS, text_height_points};
use crate::grid::Slot;
use crate::pagination::{height_or_default, page_breaks};

/// A worksheet written top to bottom: the sections write at the current row and move on, and the canvas estimates
/// each row's height and places the page breaks when the last section is done.
pub(crate) struct Canvas<'sheet> {
    worksheet: &'sheet mut Worksheet,
    row: u32,
    heights: BTreeMap<u32, f64>,
    blocks: Vec<Range<u32>>,
}

impl<'sheet> Canvas<'sheet> {
    pub(crate) const fn new(worksheet: &'sheet mut Worksheet) -> Self {
        Self {
            worksheet,
            row: 0,
            heights: BTreeMap::new(),
            blocks: Vec::new(),
        }
    }

    pub(crate) const fn row(&self) -> u32 {
        self.row
    }

    /// The last row of `row_count` rows that start at the current one.
    pub(crate) fn last_row_of(&self, row_count: usize) -> u32 {
        u32::try_from(row_count.saturating_sub(1)).map_or(self.row, |extra| self.row + extra)
    }

    pub(crate) const fn next_row(&mut self) {
        self.row += 1;
    }

    /// Writes `text` into the current row.
    pub(crate) fn put(&mut self, slot: Slot, text: &str) -> Result<(), XlsxError> {
        self.put_down(self.row, slot, text)
    }

    /// Writes `text` into one cell that spans the current row down to `last_row`.
    pub(crate) fn put_down(
        &mut self,
        last_row: u32,
        slot: Slot,
        text: &str,
    ) -> Result<(), XlsxError> {
        let Slot { columns, look } = slot;
        let format = look.format();
        if self.row == last_row && columns.first == columns.last {
            self.worksheet
                .write_string_with_format(self.row, columns.first, text, &format)?;
        } else {
            self.worksheet.merge_range(
                self.row,
                columns.first,
                last_row,
                columns.last,
                text,
                &format,
            )?;
        }
        self.fit(last_row, text, slot);
        Ok(())
    }

    /// Sets the height of the current row, for a row of one line whose look the text estimate does not know.
    pub(crate) fn set_height(&mut self, points: f64) {
        self.heights.insert(self.row, points);
    }

    /// Marks the rows written by `write` as one block that stays on one page.
    pub(crate) fn block<T>(
        &mut self,
        write: impl FnOnce(&mut Self) -> Result<T, XlsxError>,
    ) -> Result<T, XlsxError> {
        let first = self.row;
        let written = write(self)?;
        self.blocks.push(first..self.row);
        Ok(written)
    }

    pub(crate) fn finish(self) -> Result<(), XlsxError> {
        let heights = self.heights;
        for (&row, &height) in &heights {
            self.worksheet.set_row_height(row, height)?;
        }
        let breaks = page_breaks(
            &|row| height_or_default(heights.get(&row)),
            self.row,
            &self.blocks,
        );
        self.worksheet.set_page_breaks(&breaks)?;
        Ok(())
    }

    /// A cell spanning several rows needs its text to fit in them all, so the last row grows by what is missing.
    fn fit(&mut self, last_row: u32, text: &str, slot: Slot) {
        let needed = text_height_points(text, slot.columns.width_pixels(), slot.look.font_points());
        let available: f64 = (self.row..=last_row)
            .map(|row| height_or_default(self.heights.get(&row)))
            .sum();
        if needed > available {
            let last = height_or_default(self.heights.get(&last_row));
            self.heights.insert(
                last_row,
                (last + needed - available).min(MAX_ROW_HEIGHT_POINTS),
            );
        }
    }
}
