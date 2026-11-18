//! A scratch copy of the fixture vault for a test that breaks it.

use std::fs;
use std::path::{Path, PathBuf};

use crate::{Report, validate};

#[path = "../../vault/tests/support/replace_in.rs"]
mod replace_in_support;
#[path = "../../vault/tests/support/scratch.rs"]
mod scratch;
#[path = "../../vault/tests/support/vault_copy.rs"]
mod vault_copy;

/// Fixed, so a test does not depend on the day it runs.
pub(crate) const CURRENT_MONTH: &str = "2026-11";

#[derive(Debug)]
pub(crate) struct TestVault {
    root: PathBuf,
}

impl TestVault {
    /// `test` names the copy, so tests running side by side do not share one.
    pub(crate) fn new(test: &str) -> Self {
        Self {
            root: vault_copy::copy_of_fixture_vault(test).unwrap(),
        }
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// Replaces the first `from` in the file; the test fails when the file has none.
    pub(crate) fn replace(&self, file: &str, from: &str, to: &str) -> &Self {
        replace_in_support::replace_in(&self.root, file, from, to).unwrap();
        self
    }

    pub(crate) fn write(&self, file: &str, text: &str) -> &Self {
        let path = self.root.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
        self
    }

    pub(crate) fn remove(&self, file: &str) -> &Self {
        fs::remove_file(self.root.join(file)).unwrap();
        self
    }

    pub(crate) fn rename(&self, from: &str, to: &str) -> &Self {
        let target = self.root.join(to);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::rename(self.root.join(from), target).unwrap();
        self
    }

    pub(crate) fn report(&self) -> Report {
        validate(&self.root, CURRENT_MONTH.parse().unwrap()).unwrap()
    }

    /// Every finding, written as the command line shows it.
    pub(crate) fn lines(&self) -> Vec<String> {
        self.report()
            .findings()
            .iter()
            .map(ToString::to_string)
            .collect()
    }
}

/// For a break that causes other findings too: this one must be among them.
pub(crate) fn assert_reports(vault: &TestVault, expected: &str) {
    let lines = vault.lines();
    assert!(
        lines.iter().any(|line| line == expected),
        "{expected}\nis not among\n{lines:#?}"
    );
}

impl Drop for TestVault {
    fn drop(&mut self) {
        // A copy left behind is overwritten by the next run, so a failed removal is not worth a panic in drop.
        fs::remove_dir_all(&self.root).unwrap_or_default();
    }
}
