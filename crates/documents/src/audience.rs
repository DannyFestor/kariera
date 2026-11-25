use std::borrow::Cow;

use kariera_domain::Personal;

/// Who a document is for, which decides the personal details it may print.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Audience {
    /// The agencies and employers the owner sends it to: every detail of `personal.yaml`.
    Private,
    /// Anyone, on the owner's website: the `public:` map of `personal.yaml` and nothing else.
    Public,
}

impl Audience {
    pub const ALL: [Self; 2] = [Self::Private, Self::Public];

    pub(crate) fn personal(self, personal: &Personal) -> Cow<'_, Personal> {
        match self {
            Self::Private => Cow::Borrowed(personal),
            Self::Public => Cow::Owned(personal.public.to_personal()),
        }
    }
}

#[cfg(test)]
mod tests {
    use kariera_domain::{Personal, Photo, PublicDetails, Text, Translated};

    use super::Audience;

    fn text(value: &str) -> Text {
        value.parse().unwrap()
    }

    fn every_detail() -> Personal {
        Personal {
            email: Some(text("robin@example.com")),
            phone: Some(text("+81 90 0000 0000")),
            location: Some(text("Sapporo, Japan")),
            birth_date: Some("1990-01-01".parse().unwrap()),
            nationality: Some(Translated {
                en: text("German"),
                ja: None,
                de: None,
            }),
            photo: Some(Photo(vec![1, 2, 3])),
            name_kana: Some(text("ロビン")),
            gender: Some(text("男")),
            postal_code: Some(text("001-0000")),
            address: Some(text("北海道札幌市")),
            address_kana: Some(text("ほっかいどう さっぽろし")),
            residence_status: Some(text("技術・人文知識・国際業務")),
            work_restriction: Some(text("就労制限なし")),
            residence_expiry: Some("2028-03-31".parse().unwrap()),
            residence_note: Some(text("更新申請中")),
            public: PublicDetails {
                email: Some(text("hello@example.com")),
                location: Some(text("Japan")),
            },
        }
    }

    #[test]
    fn a_private_document_prints_every_detail() {
        let personal = every_detail();

        assert_eq!(*Audience::Private.personal(&personal), personal);
    }

    #[test]
    fn a_public_document_prints_only_the_public_email_and_location() {
        assert_eq!(
            *Audience::Public.personal(&every_detail()),
            Personal {
                email: Some(text("hello@example.com")),
                location: Some(text("Japan")),
                ..Personal::default()
            }
        );
    }

    #[test]
    fn without_a_public_map_a_public_document_prints_no_detail() {
        let personal = Personal {
            public: PublicDetails::default(),
            ..every_detail()
        };

        assert_eq!(*Audience::Public.personal(&personal), Personal::default());
    }
}
