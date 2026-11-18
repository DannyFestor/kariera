//! The sheet is a grid of twelve equal columns; every cell is a span of them.

use crate::look::Look;

pub(crate) const COLUMN_COUNT: u16 = 12;
/// In Excel's character units.
pub(crate) const COLUMN_WIDTH: f64 = 7.5;

const PIXELS_PER_CHARACTER: f64 = 7.0;
const COLUMN_PADDING_PIXELS: f64 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Columns {
    pub(crate) first: u16,
    pub(crate) last: u16,
}

impl Columns {
    pub(crate) const fn new(first: u16, last: u16) -> Self {
        Self { first, last }
    }

    pub(crate) const fn one(column: u16) -> Self {
        Self::new(column, column)
    }

    pub(crate) const FULL: Self = Self::new(0, COLUMN_COUNT - 1);

    pub(crate) fn width_pixels(self) -> f64 {
        let count = f64::from(self.last - self.first + 1);
        count * COLUMN_WIDTH.mul_add(PIXELS_PER_CHARACTER, COLUMN_PADDING_PIXELS)
    }
}

/// Where a piece of text goes and what it looks like: the unit the sections lay out in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Slot {
    pub(crate) columns: Columns,
    pub(crate) look: Look,
}

impl Slot {
    pub(crate) const fn new(columns: Columns, look: Look) -> Self {
        Self { columns, look }
    }

    pub(crate) const fn looking(self, look: Look) -> Self {
        Self { look, ..self }
    }
}
