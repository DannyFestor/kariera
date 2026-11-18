//! The page, fonts, sizes and colours the DOCX files share. Lengths are twentieths of a point, font sizes half-points,
//! the units DOCX itself uses.

use docx_rs::RunFonts;

pub(crate) const PAGE_WIDTH: u32 = 11_906;
pub(crate) const PAGE_HEIGHT: u32 = 16_838;
pub(crate) const SIDE_MARGIN: i32 = 907;
pub(crate) const TEXT_WIDTH: usize = 10_092;

pub(crate) const MUTED: &str = "696969";
pub(crate) const FAINT: &str = "B4B4B4";
pub(crate) const RULE: &str = "D7D7D7";
pub(crate) const CHIP: &str = "F1F1F4";

/// Fonts are named, not embedded: docx-rs cannot embed them, and `BIZ UDPGothic` ships with Windows 10 (1809) and later.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Typeface {
    latin: &'static str,
    east_asian: &'static str,
}

impl Typeface {
    /// Calibri, which ships with Office on Windows and Mac, for Latin text; the PDF uses Inter. `BIZ UDPGothic` for
    /// Japanese glyphs (issue #8).
    pub(crate) const CV: Self = Self {
        latin: "Calibri",
        east_asian: "BIZ UDPGothic",
    };
    pub(crate) const JAPANESE: Self = Self {
        latin: "BIZ UDPGothic",
        east_asian: "BIZ UDPGothic",
    };

    pub(crate) fn run_fonts(self) -> RunFonts {
        RunFonts::new()
            .ascii(self.latin)
            .hi_ansi(self.latin)
            .east_asia(self.east_asian)
            .cs(self.latin)
    }
}

/// What differs between the documents' looks; the layout code reads it instead of constants of its own.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Palette {
    pub(crate) accent: &'static str,
    pub(crate) ink: &'static str,
}
