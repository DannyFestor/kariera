//! The schema's Enum fields: stored as a slug, shown as a label per language (schema §1.12).

use crate::Language;

pub trait FixedList: Sized + Copy + 'static {
    /// In the order of the schema's table, which is also the order `Ord` gives.
    const ALL: &'static [Self];

    fn slug(self) -> &'static str;

    #[must_use]
    fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|value| value.slug() == slug)
    }

    #[must_use]
    fn slugs() -> Vec<&'static str> {
        Self::ALL.iter().map(|value| value.slug()).collect()
    }
}

pub trait Labelled: FixedList {
    fn label(self, language: Language) -> &'static str;
}

/// Declares an enum with its slugs, so the slug table exists once and in schema order.
macro_rules! fixed_list {
    ($(#[$meta:meta])* $vis:vis $name:ident { $($variant:ident => $slug:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $vis enum $name {
            $($variant),+
        }

        impl $crate::FixedList for $name {
            const ALL: &'static [Self] = &[$(Self::$variant),+];

            fn slug(self) -> &'static str {
                match self {
                    $(Self::$variant => $slug),+
                }
            }
        }
    };
}

/// A fixed list whose labels the schema defines in English, Japanese and German, in that order.
macro_rules! labelled_list {
    ($(#[$meta:meta])* $vis:vis $name:ident {
        $($variant:ident => $slug:literal [$en:literal, $ja:literal, $de:literal]),+ $(,)?
    }) => {
        $crate::fixed_list::fixed_list!($(#[$meta])* $vis $name { $($variant => $slug),+ });

        impl $crate::Labelled for $name {
            fn label(self, language: $crate::Language) -> &'static str {
                let [en, ja, de] = match self {
                    $(Self::$variant => [$en, $ja, $de]),+
                };
                match language {
                    $crate::Language::English => en,
                    $crate::Language::Japanese => ja,
                    $crate::Language::German => de,
                }
            }
        }
    };
}

pub(crate) use {fixed_list, labelled_list};

#[cfg(test)]
mod tests {
    use super::{FixedList, Labelled};
    use crate::Language;

    labelled_list!(pub(super) Colour {
        Red => "red" ["Red", "赤", "Rot"],
        LightBlue => "light-blue" ["Light blue", "水色", "Hellblau"],
    });

    #[test]
    fn a_slug_names_its_value() {
        assert_eq!(Colour::from_slug("light-blue"), Some(Colour::LightBlue));
    }

    #[test]
    fn an_unknown_slug_names_nothing() {
        assert_eq!(Colour::from_slug("Light blue"), None);
    }

    #[test]
    fn slugs_are_listed_in_declaration_order() {
        assert_eq!(Colour::slugs(), ["red", "light-blue"]);
    }

    #[test]
    fn each_language_gets_its_own_label() {
        let labels = Language::ALL.map(|language| Colour::LightBlue.label(language));

        assert_eq!(labels, ["Light blue", "水色", "Hellblau"]);
    }
}
