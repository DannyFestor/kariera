use std::io;
use std::path::PathBuf;

use kariera_domain::{
    Fingerprint, FixedList, NoteName, NoteType, Translatable, TranslationLanguage,
};
use thiserror::Error;

use crate::fingerprint::fingerprint;
use crate::names::nfc;
use crate::translated_from::{self, EditError};
use crate::write::replace_file;
use crate::{NoteFile, Vault};

/// One language of one note that carries `translated_from` (schema §1.6.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Translation {
    pub name: NoteName,
    /// Relative to the vault root.
    pub path: PathBuf,
    pub language: TranslationLanguage,
    /// Empty: never translated.
    pub stored: Option<Fingerprint>,
    pub current: Fingerprint,
}

impl Translation {
    #[must_use]
    pub fn is_stale(&self) -> bool {
        self.stored.as_ref() != Some(&self.current)
    }
}

#[derive(Debug, Error)]
pub enum MarkError {
    #[error("no note is named {0:?}")]
    NoSuchNote(String),
    #[error("{name} is a {} note, which has no translations", note_type.slug())]
    NotTranslatable { name: NoteName, note_type: NoteType },
    #[error("cannot change {}: it {source}", path.display())]
    Edit { path: PathBuf, source: EditError },
    #[error("cannot write {}: {source}", path.display())]
    Write { path: PathBuf, source: io::Error },
}

impl Vault {
    /// Every language of every note that carries `translated_from`, in the order of the notes' paths.
    #[must_use]
    pub fn translations(&self) -> Vec<Translation> {
        self.translatable_notes()
            .flat_map(|(note, translatable)| translations_of(note, translatable))
            .collect()
    }

    /// Writes the note's current fingerprint into `translated_from` for `language`, and nothing else in the file.
    /// `name` is compared in NFC (schema §1.1).
    pub fn mark(
        &self,
        name: &str,
        language: TranslationLanguage,
    ) -> Result<Fingerprint, MarkError> {
        let name = nfc(name);
        let note = self
            .notes()
            .iter()
            .find(|note| note.name.as_str() == name)
            .ok_or(MarkError::NoSuchNote(name))?;
        let translatable = self
            .career()
            .translatable(note.note_type, &note.name)
            .ok_or_else(|| MarkError::NotTranslatable {
                name: note.name.clone(),
                note_type: note.note_type,
            })?;
        let current = fingerprint(&translatable.english_content());
        let text = translated_from::set(&note.text, language, &current).map_err(|source| {
            MarkError::Edit {
                path: note.path.clone(),
                source,
            }
        })?;
        replace_file(&self.root().join(&note.path), &text).map_err(|source| MarkError::Write {
            path: note.path.clone(),
            source,
        })?;
        Ok(current)
    }

    fn translatable_notes(&self) -> impl Iterator<Item = (&NoteFile, &dyn Translatable)> {
        self.notes().iter().filter_map(|note| {
            self.career()
                .translatable(note.note_type, &note.name)
                .map(|translatable| (note, translatable))
        })
    }
}

fn translations_of(note: &NoteFile, translatable: &dyn Translatable) -> Vec<Translation> {
    let current = fingerprint(&translatable.english_content());
    TranslationLanguage::ALL
        .into_iter()
        .map(|language| Translation {
            name: note.name.clone(),
            path: note.path.clone(),
            language,
            stored: translatable.translated_from().get(language).cloned(),
            current: current.clone(),
        })
        .collect()
}
