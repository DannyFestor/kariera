//! Excel does not fit a row to its wrapped text when it opens a generated file, so every row height is estimated
//! here from the text and the width of the cells it sits in.

const POINTS_TO_PIXELS: f64 = 96.0 / 72.0;
const LINE_SPACING: f64 = 1.45;
const PADDING_POINTS: f64 = 5.0;
const NARROW_GLYPH_EM: f64 = 0.55;
const CELL_INSET_PIXELS: f64 = 8.0;
const SMALLEST_USABLE_WIDTH_PIXELS: f64 = 10.0;
pub(crate) const DEFAULT_ROW_HEIGHT_POINTS: f64 = 15.0;
/// Excel refuses a taller row.
pub(crate) const MAX_ROW_HEIGHT_POINTS: f64 = 409.0;

pub(crate) fn text_height_points(text: &str, width_pixels: f64, font_points: f64) -> f64 {
    let lines = line_count(text, width_pixels, font_points);
    (lines * font_points)
        .mul_add(LINE_SPACING, PADDING_POINTS)
        .clamp(DEFAULT_ROW_HEIGHT_POINTS, MAX_ROW_HEIGHT_POINTS)
}

fn line_count(text: &str, width_pixels: f64, font_points: f64) -> f64 {
    let usable = (width_pixels - CELL_INSET_PIXELS).max(SMALLEST_USABLE_WIDTH_PIXELS);
    text.split('\n')
        .map(|line| {
            (line_width_pixels(line, font_points) / usable)
                .ceil()
                .max(1.0)
        })
        .sum()
}

fn line_width_pixels(line: &str, font_points: f64) -> f64 {
    let em = font_points * POINTS_TO_PIXELS;
    line.chars()
        .map(|glyph| {
            if is_wide(glyph) {
                em
            } else {
                em * NARROW_GLYPH_EM
            }
        })
        .sum()
}

/// East Asian wide and fullwidth glyphs, which are as wide as they are tall.
const fn is_wide(glyph: char) -> bool {
    matches!(glyph,
        '\u{1100}'..='\u{115F}'
        | '\u{2E80}'..='\u{A4CF}'
        | '\u{AC00}'..='\u{D7A3}'
        | '\u{F900}'..='\u{FAFF}'
        | '\u{FE30}'..='\u{FE6F}'
        | '\u{FF00}'..='\u{FF60}'
        | '\u{FFE0}'..='\u{FFE6}')
}
