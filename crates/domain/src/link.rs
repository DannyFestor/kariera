use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::str::FromStr;

use crate::ValueError;

/// A note's file name without `.md`, unique across the vault (schema §1.1). The reader normalizes it to Unicode NFC
/// before parsing, because macOS writes NFD.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoteName(String);

const FORBIDDEN_IN_NAMES: [char; 13] = [
    '#', '^', '[', ']', '|', '\\', '/', ':', '*', '?', '"', '<', '>',
];

impl NoteName {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for NoteName {
    type Err = ValueError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        if name.is_empty() {
            return Err(ValueError::Blank);
        }
        if let Some(forbidden) = name.chars().find(|c| FORBIDDEN_IN_NAMES.contains(c)) {
            return Err(ValueError::ForbiddenCharacter(forbidden));
        }
        if name.starts_with([' ', '.']) || name.ends_with([' ', '.']) {
            return Err(ValueError::SpaceOrDotAtEdge);
        }
        Ok(Self(name.to_owned()))
    }
}

impl fmt::Display for NoteName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Debug for NoteName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

/// A link to the note named `target`, which is a `T` (schema §1.4). The vault reader checks both when it reads the
/// vault.
pub struct Link<T> {
    target: NoteName,
    // fn() -> T: a Link neither owns a T nor needs T to be Send, Sync, Clone or Debug.
    note_type: PhantomData<fn() -> T>,
}

impl<T> Link<T> {
    #[must_use]
    pub const fn new(target: NoteName) -> Self {
        Self {
            target,
            note_type: PhantomData,
        }
    }

    #[must_use]
    pub const fn target(&self) -> &NoteName {
        &self.target
    }
}

// Derives would demand the same trait of T, which the PhantomData does not need.
impl<T> Clone for Link<T> {
    fn clone(&self) -> Self {
        Self::new(self.target.clone())
    }
}

impl<T> PartialEq for Link<T> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
    }
}

impl<T> Eq for Link<T> {}

impl<T> PartialOrd for Link<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Link<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.target.cmp(&other.target)
    }
}

impl<T> Hash for Link<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
    }
}

impl<T> fmt::Debug for Link<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "[[{}]]", self.target)
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{DefaultHasher, Hash, Hasher};

    use super::{Link, NoteName};
    use crate::ValueError;

    fn name(text: &str) -> NoteName {
        text.parse().unwrap()
    }

    #[test]
    fn a_plain_file_name_is_a_note_name() {
        assert_eq!(name("2025-06 Acme").as_str(), "2025-06 Acme");
        assert_eq!(name("C Sharp").to_string(), "C Sharp");
        assert_eq!(format!("{:?}", name("Acme")), "\"Acme\"");
    }

    #[test]
    fn a_name_with_a_forbidden_character_is_refused() {
        for forbidden in [
            '#', '^', '[', ']', '|', '\\', '/', ':', '*', '?', '"', '<', '>',
        ] {
            assert_eq!(
                format!("C{forbidden}").parse::<NoteName>(),
                Err(ValueError::ForbiddenCharacter(forbidden))
            );
        }
    }

    #[test]
    fn a_name_starting_or_ending_with_a_space_or_dot_is_refused() {
        for text in [" Acme", "Acme ", ".Acme", "Acme."] {
            assert_eq!(
                text.parse::<NoteName>(),
                Err(ValueError::SpaceOrDotAtEdge),
                "{text}"
            );
        }
    }

    #[test]
    fn an_empty_name_is_refused() {
        assert_eq!("".parse::<NoteName>(), Err(ValueError::Blank));
    }

    #[test]
    fn links_compare_and_print_by_their_target() {
        let acme = Link::<()>::new(name("Acme"));
        let globex = Link::<()>::new(name("Globex"));

        assert_eq!(acme.clone(), acme);
        assert_ne!(acme, globex);
        assert!(acme < globex);
        assert_eq!(acme.target(), &name("Acme"));
        assert_eq!(format!("{acme:?}"), "[[Acme]]");
    }

    #[test]
    fn a_link_hashes_as_its_target_name() {
        let hash = |value: &dyn Fn(&mut DefaultHasher)| {
            let mut hasher = DefaultHasher::new();
            value(&mut hasher);
            hasher.finish()
        };

        let link = hash(&|hasher| Link::<()>::new(name("Acme")).hash(hasher));
        let target = hash(&|hasher| name("Acme").hash(hasher));

        assert_eq!(link, target);
    }
}
