use thiserror::Error;

/// Why a written value does not fit its field type (schema §1.2). Each message completes "the field …".
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValueError {
    #[error("must not be blank")]
    Blank,
    #[error("must be a single line")]
    MultiLine,
    #[error("must not start or end with whitespace")]
    Untrimmed,
    #[error("must start with https://")]
    NotHttps,
    #[error("must be a month written YYYY-MM, such as 2024-05")]
    NotAMonth,
    #[error("must be a date written YYYY-MM-DD, such as 2026-03-04")]
    NotADate,
    #[error("must be a number of hours with at most 2 decimals, such as 152.5")]
    NotHours,
    #[error("must be a whole number from {min} to {max}")]
    OutOfRange { min: u32, max: u32 },
    #[error("must not contain {0:?}")]
    ForbiddenCharacter(char),
    #[error("must not start or end with a space or a dot")]
    SpaceOrDotAtEdge,
    #[error("must be a path relative to the vault, with / separators and no ..")]
    NotRelativePath,
}
