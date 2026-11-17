use std::collections::BTreeMap;
use std::path::Path;

use kariera_domain::{Career, Contract, Engagement, Link, Month, Note, NoteName, Organization};
use kariera_vault::{LoadError, NoteFile, Stray, Vault};

/// A loaded vault with what the checks look things up by: the file of every note and the files that are notes in
/// the wrong place.
#[derive(Debug)]
pub(crate) struct Context<'a> {
    vault: &'a Vault,
    strays: Vec<Stray>,
    complete: bool,
    current_month: Month,
    files: BTreeMap<&'a NoteName, &'a NoteFile>,
}

/// A note of type `T` with the file it was read from.
#[derive(Debug)]
pub(crate) struct Found<'a, T> {
    pub(crate) name: &'a NoteName,
    pub(crate) file: &'a NoteFile,
    pub(crate) note: &'a T,
}

// A derive would demand `T: Clone`, which a reference does not need.
impl<T> Clone for Found<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Found<'_, T> {}

impl<'a> Context<'a> {
    /// `complete` is true when every note of the vault was read into `vault`.
    pub(crate) fn new(
        vault: &'a Vault,
        complete: bool,
        current_month: Month,
    ) -> Result<Self, LoadError> {
        Ok(Self {
            vault,
            strays: vault.strays()?,
            complete,
            current_month,
            files: vault
                .notes()
                .iter()
                .map(|file| (&file.name, file))
                .collect(),
        })
    }

    /// Whether a note missing from the vault can be told from a note that was never written.
    pub(crate) const fn is_complete(&self) -> bool {
        self.complete
    }

    pub(crate) const fn current_month(&self) -> Month {
        self.current_month
    }

    pub(crate) fn root(&self) -> &Path {
        self.vault.root()
    }

    pub(crate) const fn career(&self) -> &'a Career {
        self.vault.career()
    }

    pub(crate) fn files(&self) -> &'a [NoteFile] {
        self.vault.notes()
    }

    pub(crate) fn strays(&self) -> &[Stray] {
        &self.strays
    }

    pub(crate) fn file(&self, name: &NoteName) -> Option<&'a NoteFile> {
        self.files.get(name).copied()
    }

    /// Every note of type `T`, in the order of their names.
    pub(crate) fn notes<T: Note + 'a>(&self) -> impl Iterator<Item = Found<'a, T>> + '_ {
        T::all(self.career())
            .iter()
            .filter_map(|(name, note)| self.found(name, note))
    }

    /// The note a link names; `None` when the link resolves to nothing, which the vault reader has reported.
    pub(crate) fn get<T: Note + 'a>(&self, link: &Link<T>) -> Option<Found<'a, T>> {
        T::all(self.career())
            .get_key_value(link.target())
            .and_then(|(name, note)| self.found(name, note))
    }

    /// Every Engagement that lists the Contract in `contracts`; a valid vault has exactly one (schema §2.3).
    pub(crate) fn engagements_listing(&self, contract: &NoteName) -> Vec<Found<'a, Engagement>> {
        self.notes::<Engagement>()
            .filter(|engagement| {
                engagement
                    .note
                    .contracts
                    .iter()
                    .any(|listed| listed.target() == contract)
            })
            .collect()
    }

    /// The Engagement the Contract belongs to, when exactly one lists it.
    pub(crate) fn engagement_of(
        &self,
        contract: &Found<'_, Contract>,
    ) -> Option<Found<'a, Engagement>> {
        let mut listing = self.engagements_listing(contract.name);
        match (listing.pop(), listing.is_empty()) {
            (Some(engagement), true) => Some(engagement),
            (Some(_) | None, _) => None,
        }
    }

    fn found<T>(&self, name: &'a NoteName, note: &'a T) -> Option<Found<'a, T>> {
        self.file(name).map(|file| Found { name, file, note })
    }
}

/// Exactly one of `client` and `employer` is set (schema §2.2).
pub(crate) fn client_link(engagement: &Engagement) -> Option<&Link<Organization>> {
    engagement.client.as_ref().or(engagement.employer.as_ref())
}
