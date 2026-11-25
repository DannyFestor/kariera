use std::fmt;

use crate::{Date, Text, Translated};

/// The owner's contact and personal details from `~/.config/kariera/personal.yaml`. They never live in the vault,
/// so documents get them from outside it; every field is optional.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Personal {
    pub email: Option<Text>,
    pub phone: Option<Text>,
    pub location: Option<Text>,
    pub birth_date: Option<Date>,
    /// Every document prints its language's entry, else the English one.
    pub nationality: Option<Translated<Text>>,
    pub photo: Option<Photo>,
    /// The name in katakana, for the ふりがな of the 履歴書.
    pub name_kana: Option<Text>,
    pub gender: Option<Text>,
    pub postal_code: Option<Text>,
    pub address: Option<Text>,
    pub address_kana: Option<Text>,
    /// The 在留資格 of a foreign resident of Japan.
    pub residence_status: Option<Text>,
    /// What `residence_status` allows: 就労制限の有無.
    pub work_restriction: Option<Text>,
    pub residence_expiry: Option<Date>,
    /// Anything else about the residence, such as an application in progress.
    pub residence_note: Option<Text>,
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
