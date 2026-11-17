use std::collections::BTreeMap;
use std::fmt::Display;
use std::str::FromStr;

use kariera_domain::FixedList;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ArgumentError {
    #[error("expected log, project or engagement")]
    MissingNoteType,
    #[error("{0:?} is not log, project or engagement")]
    UnknownNoteType(String),
    #[error("unexpected argument {0:?}, flags look like --name value")]
    UnexpectedArgument(String),
    #[error("--{0} needs a value")]
    MissingValue(String),
    #[error("--{0} is given twice")]
    RepeatedFlag(String),
    #[error("unknown flag --{0}")]
    UnknownFlag(String),
    #[error("--{0} is required")]
    MissingFlag(&'static str),
    #[error("--{flag} {value:?}: {reason}")]
    InvalidValue {
        flag: &'static str,
        value: String,
        reason: String,
    },
    #[error("an engagement needs exactly one of --client and --employer")]
    ClientXorEmployer,
}

/// The `--name value` and `--name=value` pairs of a command line. A flag a note type does not read is left over,
/// which `finish` reports.
#[derive(Debug)]
pub(crate) struct Flags(BTreeMap<String, String>);

impl Flags {
    pub(crate) fn parse(
        arguments: impl IntoIterator<Item = String>,
    ) -> Result<Self, ArgumentError> {
        let mut arguments = arguments.into_iter();
        let mut flags = BTreeMap::new();
        while let Some(argument) = arguments.next() {
            let flag = argument
                .strip_prefix("--")
                .ok_or_else(|| ArgumentError::UnexpectedArgument(argument.clone()))?;
            let (name, value) = match flag.split_once('=') {
                Some((name, value)) => (name, value.to_owned()),
                None => (
                    flag,
                    arguments
                        .next()
                        .ok_or_else(|| ArgumentError::MissingValue(flag.to_owned()))?,
                ),
            };
            if flags.insert(name.to_owned(), value).is_some() {
                return Err(ArgumentError::RepeatedFlag(name.to_owned()));
            }
        }
        Ok(Self(flags))
    }

    pub(crate) fn required<T>(&mut self, flag: &'static str) -> Result<T, ArgumentError>
    where
        T: FromStr,
        T::Err: Display,
    {
        let value = self
            .0
            .remove(flag)
            .ok_or(ArgumentError::MissingFlag(flag))?;
        value
            .parse()
            .map_err(|error: T::Err| invalid(flag, value, error))
    }

    pub(crate) fn required_slug<T: FixedList>(
        &mut self,
        flag: &'static str,
    ) -> Result<T, ArgumentError> {
        let value = self
            .0
            .remove(flag)
            .ok_or(ArgumentError::MissingFlag(flag))?;
        T::from_slug(&value).ok_or_else(|| {
            let reason = format!("expected one of {}", T::slugs().join(", "));
            invalid(flag, value, reason)
        })
    }

    pub(crate) fn optional<T>(&mut self, flag: &'static str) -> Result<Option<T>, ArgumentError>
    where
        T: FromStr,
        T::Err: Display,
    {
        self.0
            .remove(flag)
            .map(|value| {
                value
                    .parse()
                    .map_err(|error: T::Err| invalid(flag, value, error))
            })
            .transpose()
    }

    pub(crate) fn finish(self) -> Result<(), ArgumentError> {
        self.0
            .into_keys()
            .next()
            .map_or(Ok(()), |flag| Err(ArgumentError::UnknownFlag(flag)))
    }
}

fn invalid(flag: &'static str, value: String, reason: impl Display) -> ArgumentError {
    ArgumentError::InvalidValue {
        flag,
        value,
        reason: reason.to_string(),
    }
}
