use std::collections::BTreeMap;

use kariera_domain::{FixedList, Link, Note, NoteName, NoteType};
use unicode_normalization::UnicodeNormalization;

/// Every note name of the vault with its type, for resolving links. The type is unknown for a note whose `type`
/// could not be read; links to it resolve, so its own problem is the one reported.
#[derive(Debug, Default)]
pub(crate) struct Names(BTreeMap<NoteName, Option<NoteType>>);

impl Names {
    pub(crate) fn insert(&mut self, name: NoteName, note_type: Option<NoteType>) {
        self.0.insert(name, note_type);
    }

    pub(crate) fn link<T: Note>(&self, name: NoteName) -> Result<Link<T>, String> {
        match self.0.get(&name) {
            None => Err(format!("links to [[{name}]], but no note has that name")),
            Some(Some(found)) if *found != T::TYPE => Err(format!(
                "links to [[{name}]], whose type is {}, not {}",
                found.slug(),
                T::TYPE.slug()
            )),
            Some(Some(_) | None) => Ok(Link::new(name)),
        }
    }
}

/// Names and link targets are compared in NFC (schema §1.1): macOS and Syncthing also write NFD.
pub(crate) fn nfc(text: &str) -> String {
    text.nfc().collect()
}

#[cfg(test)]
mod tests {
    use kariera_domain::{Link, NoteName, NoteType, Organization, Skill};

    use super::{Names, nfc};

    fn name(text: &str) -> NoteName {
        text.parse().unwrap()
    }

    fn names() -> Names {
        let mut names = Names::default();
        names.insert(name("Acme"), Some(NoteType::Organization));
        names.insert(name("Broken"), None);
        names
    }

    #[test]
    fn a_link_to_a_note_of_its_type_resolves() {
        assert_eq!(
            names().link::<Organization>(name("Acme")),
            Ok(Link::new(name("Acme")))
        );
    }

    #[test]
    fn a_link_to_a_note_of_another_type_does_not_resolve() {
        assert_eq!(
            names().link::<Skill>(name("Acme")),
            Err("links to [[Acme]], whose type is organization, not skill".to_owned())
        );
    }

    #[test]
    fn a_link_to_no_note_does_not_resolve() {
        assert_eq!(
            names().link::<Skill>(name("Rust")),
            Err("links to [[Rust]], but no note has that name".to_owned())
        );
    }

    #[test]
    fn a_link_to_a_note_of_unknown_type_resolves() {
        assert_eq!(
            names().link::<Skill>(name("Broken")),
            Ok(Link::new(name("Broken")))
        );
    }

    #[test]
    fn nfd_text_becomes_nfc() {
        assert_eq!(nfc("Mu\u{308}ller ガ"), "Müller ガ");
        assert_eq!(nfc("Müller"), "Müller");
    }
}
