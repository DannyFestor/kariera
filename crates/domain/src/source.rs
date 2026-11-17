use std::fmt;
use std::str::FromStr;

use crate::fixed_list::fixed_list;
use crate::{Text, ValueError};

/// The evidence a fact rests on (schema §1.7). `path` is empty exactly when `kind` is `recalled`, and `note` is
/// required for `recalled`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Source {
    pub kind: SourceKind,
    pub path: Option<SourcePath>,
    pub note: Option<Text>,
}

fixed_list!(pub SourceKind {
    WorkReport => "work-report",
    Invoice => "invoice",
    Contract => "contract",
    PurchaseOrder => "purchase-order",
    Timesheet => "timesheet",
    SkillSheet => "skill-sheet",
    Cv => "cv",
    TrackerDigest => "tracker-digest",
    Certificate => "certificate",
    Recalled => "recalled",
});

/// Relative to the vault root, `/` separators, no leading `/`, no `..` (schema §1.7). Whether the file exists is a
/// question for the file system, not for this type.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourcePath(Text);

impl SourcePath {
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl FromStr for SourcePath {
    type Err = ValueError;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        let text = path.parse::<Text>()?;
        let is_relative = !path.starts_with('/')
            && !path.contains('\\')
            && path
                .split('/')
                .all(|segment| segment != ".." && !segment.is_empty());
        is_relative
            .then_some(Self(text))
            .ok_or(ValueError::NotRelativePath)
    }
}

impl fmt::Debug for SourcePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::SourcePath;
    use crate::ValueError;

    #[test]
    fn a_relative_path_is_a_source_path() {
        let path = "inputs/Acme/2025-06 invoice.pdf"
            .parse::<SourcePath>()
            .unwrap();

        assert_eq!(path.as_str(), "inputs/Acme/2025-06 invoice.pdf");
        assert_eq!(format!("{path:?}"), "\"inputs/Acme/2025-06 invoice.pdf\"");
    }

    #[test]
    fn a_path_that_could_leave_the_vault_is_refused() {
        for path in [
            "/inputs/a.pdf",
            "../a.pdf",
            "inputs/../a.pdf",
            "inputs\\a.pdf",
            "inputs//a.pdf",
            "a/",
        ] {
            assert_eq!(
                path.parse::<SourcePath>(),
                Err(ValueError::NotRelativePath),
                "{path}"
            );
        }
    }

    #[test]
    fn a_source_path_is_text_first() {
        assert_eq!(" a.pdf".parse::<SourcePath>(), Err(ValueError::Untrimmed));
    }
}
