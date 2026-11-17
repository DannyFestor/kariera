use crate::finding::{Finding, Severity};

/// Everything `validate` found. Errors make a vault invalid; warnings do not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    findings: Vec<Finding>,
    notes_checked: usize,
    complete: bool,
}

impl Report {
    pub(crate) fn new(mut findings: Vec<Finding>, notes_checked: usize, complete: bool) -> Self {
        findings.sort();
        Self {
            findings,
            notes_checked,
            complete,
        }
    }

    /// In the order of the notes' paths, then of their lines.
    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    #[must_use]
    pub fn errors(&self) -> usize {
        self.count(Severity::Error)
    }

    #[must_use]
    pub fn warnings(&self) -> usize {
        self.count(Severity::Warning)
    }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.errors() > 0
    }

    /// The notes the rules ran on: those that the vault reader could read.
    #[must_use]
    pub const fn notes_checked(&self) -> usize {
        self.notes_checked
    }

    /// False when some notes could not be read. The rules about a single note then ran on the others only, and
    /// the rules that need every note (a Contract listed by exactly one Engagement, exactly one Profile) did not
    /// run.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.complete
    }

    fn count(&self, severity: Severity) -> usize {
        self.findings
            .iter()
            .filter(|finding| finding.severity == severity)
            .count()
    }
}
