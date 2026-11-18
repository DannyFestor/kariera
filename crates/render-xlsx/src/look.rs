use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder};

/// Named for what a cell is, not for how it is drawn, so the sections never mention colours or sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Look {
    Band,
    BandRight,
    Key,
    Body,
    BodyRight,
    BodySmall,
    SkillCategory,
    ProjectNumber,
    ProjectTitle,
    ProjectPeriod,
    FactLabel,
    Fact,
    PhaseDone,
    PhasePending,
    TechnologyLabel,
    SectionLabel,
}

/// An XLSX cannot embed fonts, so the sheet names the family the PDF embeds. It ships with Windows.
const FONT_FAMILY: &str = "BIZ UDPゴシック";
const BASE_FONT_POINTS: f64 = 9.0;
const SMALL_FONT_POINTS: f64 = 8.0;
const TITLE_FONT_POINTS: f64 = 10.0;
const BAND_FONT_POINTS: f64 = 12.0;

const INK: u32 = 0x003B_2F5C;
const ACCENT: u32 = 0x00D9_D2EA;
const PALE: u32 = 0x00F4_F1F9;
const RULE: u32 = 0x008A_7FA8;
const WHITE: u32 = 0x00FF_FFFF;
const PHASE_PENDING_TEXT: u32 = 0x00AA_AAAA;

impl Look {
    pub(crate) const fn font_points(self) -> f64 {
        match self {
            Self::Band => BAND_FONT_POINTS,
            Self::ProjectTitle => TITLE_FONT_POINTS,
            Self::BodySmall
            | Self::SkillCategory
            | Self::PhaseDone
            | Self::PhasePending
            | Self::TechnologyLabel => SMALL_FONT_POINTS,
            Self::BandRight
            | Self::Key
            | Self::Body
            | Self::BodyRight
            | Self::ProjectNumber
            | Self::ProjectPeriod
            | Self::FactLabel
            | Self::Fact
            | Self::SectionLabel => BASE_FONT_POINTS,
        }
    }

    pub(crate) fn format(self) -> Format {
        let base = Format::new()
            .set_font_name(FONT_FAMILY)
            .set_font_size(self.font_points())
            .set_text_wrap()
            .set_align(FormatAlign::Top);
        match self {
            Self::Band | Self::BandRight => self.band(base),
            Self::Key | Self::Body | Self::BodyRight | Self::BodySmall | Self::SkillCategory => {
                self.table_cell(base)
            }
            Self::ProjectNumber | Self::ProjectTitle | Self::ProjectPeriod => {
                self.project_band(base)
            }
            Self::FactLabel | Self::TechnologyLabel | Self::SectionLabel => {
                base.set_bold().set_font_color(Color::RGB(INK))
            }
            Self::Fact => base,
            Self::PhaseDone => phase(base, INK, WHITE),
            Self::PhasePending => phase(base, PALE, PHASE_PENDING_TEXT),
        }
    }

    fn band(self, base: Format) -> Format {
        let centred = base.set_align(FormatAlign::VerticalCenter);
        if self == Self::BandRight {
            filled(centred.set_align(FormatAlign::Right), INK, WHITE)
        } else {
            filled(centred.set_bold(), INK, WHITE)
        }
    }

    fn table_cell(self, base: Format) -> Format {
        if self == Self::Key {
            ruled(filled(base.set_bold(), PALE, INK))
        } else if self == Self::BodyRight {
            ruled(base.set_align(FormatAlign::Right))
        } else {
            ruled(base)
        }
    }

    fn project_band(self, base: Format) -> Format {
        let centred = base.set_align(FormatAlign::VerticalCenter);
        if self == Self::ProjectPeriod {
            filled(centred.set_align(FormatAlign::Right), ACCENT, INK)
        } else {
            filled(centred.set_bold(), ACCENT, INK)
        }
    }
}

fn filled(format: Format, background: u32, text: u32) -> Format {
    format
        .set_background_color(Color::RGB(background))
        .set_font_color(Color::RGB(text))
}

fn ruled(format: Format) -> Format {
    format
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::RGB(RULE))
}

/// The strip of phases: a cell filled in dark is a phase the Project covered.
fn phase(format: Format, background: u32, text: u32) -> Format {
    filled(format.set_align(FormatAlign::Center), background, text)
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::RGB(WHITE))
}
