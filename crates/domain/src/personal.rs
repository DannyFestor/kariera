use std::fmt;

use crate::{Date, Text};

/// The owner's contact and personal details from `~/.config/kariera/personal.yaml`. They never live in the vault,
/// so documents get them from outside it; every field is optional.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Personal {
    pub email: Option<Text>,
    pub phone: Option<Text>,
    pub location: Option<Text>,
    pub birth_date: Option<Date>,
    pub nationality: Option<Text>,
    pub photo: Option<Photo>,
}

/// The bytes of an image file, in a format the renderer reads (PNG, JPEG, GIF, SVG or WebP for PDF).
#[derive(Clone, PartialEq, Eq)]
pub struct Photo(pub Vec<u8>);

// The size only: a snapshot holding the image's bytes would be unreadable.
impl fmt::Debug for Photo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Photo({} bytes)", self.0.len())
    }
}

#[cfg(test)]
mod tests {
    use super::Photo;

    #[test]
    fn a_photo_debugs_as_its_size() {
        assert_eq!(format!("{:?}", Photo(vec![1, 2, 3])), "Photo(3 bytes)");
    }
}
