//! The photo of the German CV as a picture in the file.

use std::io::Cursor;

use docx_rs::Pic;
use image::{GenericImageView, ImageFormat};
use kariera_domain::Photo;

use crate::RenderError;

const EMU_PER_MM: u32 = 36_000;
const BOX_WIDTH_MM: u32 = 24;
const BOX_HEIGHT_MM: u32 = 30;
/// Fixed so that the same photo gives the same bytes: docx-rs would number it from a counter of the whole process.
const PICTURE_ID: &str = "rIdImage1";

/// The picture fills as much of a 24 by 30 mm box as its proportions allow.
pub(crate) fn picture(photo: &Photo) -> Result<Pic, RenderError> {
    let decoded = image::load_from_memory(&photo.0).map_err(RenderError::Photo)?;
    let (width, height) = decoded.dimensions();
    let mut png = Cursor::new(Vec::new());
    decoded
        .write_to(&mut png, ImageFormat::Png)
        .map_err(RenderError::Photo)?;
    let (shown_width, shown_height) = fitted(width, height);
    Ok(Pic::new_with_dimensions(png.into_inner(), width, height)
        .id(PICTURE_ID)
        .size(shown_width, shown_height))
}

/// The size in EMU, the unit DOCX draws in.
fn fitted(width: u32, height: u32) -> (u32, u32) {
    let (box_width, box_height) = (BOX_WIDTH_MM * EMU_PER_MM, BOX_HEIGHT_MM * EMU_PER_MM);
    let (width, height) = (u64::from(width.max(1)), u64::from(height.max(1)));
    let wider_than_box = width * u64::from(box_height) > height * u64::from(box_width);
    let (fitted_width, fitted_height) = if wider_than_box {
        (u64::from(box_width), u64::from(box_width) * height / width)
    } else {
        (
            u64::from(box_height) * width / height,
            u64::from(box_height),
        )
    };
    (
        u32::try_from(fitted_width).unwrap_or(box_width),
        u32::try_from(fitted_height).unwrap_or(box_height),
    )
}

#[cfg(test)]
mod tests {
    use super::fitted;

    #[test]
    fn a_portrait_photo_of_the_boxs_proportions_fills_the_box() {
        assert_eq!(fitted(240, 300), (864_000, 1_080_000));
    }

    #[test]
    fn a_landscape_photo_is_as_wide_as_the_box() {
        assert_eq!(fitted(400, 200), (864_000, 432_000));
    }

    #[test]
    fn a_tall_photo_is_as_high_as_the_box() {
        assert_eq!(fitted(100, 400), (270_000, 1_080_000));
    }
}
