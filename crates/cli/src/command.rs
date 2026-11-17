use crate::translations::TranslationsCommand;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    Help,
    Version,
    Translations(TranslationsCommand),
    Unknown(String),
    /// A known command given arguments it cannot use; the text says what is wrong.
    Misused(String),
}

impl Command {
    pub(crate) fn parse(arguments: impl IntoIterator<Item = String>) -> Self {
        let mut arguments = arguments.into_iter();
        arguments.next().map_or(Self::Help, |command| {
            Self::from_argument(command, arguments)
        })
    }

    fn from_argument(argument: String, rest: impl Iterator<Item = String>) -> Self {
        match argument.as_str() {
            "help" | "--help" | "-h" => Self::Help,
            "--version" | "-V" => Self::Version,
            "translations" => {
                TranslationsCommand::parse(rest).map_or_else(Self::Misused, Self::Translations)
            }
            _ => Self::Unknown(argument),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Command;
    use crate::translations::TranslationsCommand;

    fn parse(arguments: &[&str]) -> Command {
        Command::parse(arguments.iter().map(ToString::to_string))
    }

    #[test]
    fn no_arguments_ask_for_help() {
        assert_eq!(parse(&[]), Command::Help);
    }

    #[test]
    fn every_help_spelling_asks_for_help() {
        for spelling in ["help", "--help", "-h"] {
            assert_eq!(parse(&[spelling]), Command::Help, "{spelling}");
        }
    }

    #[test]
    fn every_version_spelling_asks_for_the_version() {
        for spelling in ["--version", "-V"] {
            assert_eq!(parse(&[spelling]), Command::Version, "{spelling}");
        }
    }

    #[test]
    fn translations_takes_its_options() {
        assert_eq!(
            parse(&["translations", "--stale"]),
            Command::Translations(TranslationsCommand::Stale)
        );
    }

    #[test]
    fn translations_with_options_it_cannot_use_is_misused() {
        assert_eq!(
            parse(&["translations", "--all"]),
            Command::Misused("unknown argument \"--all\"".to_owned())
        );
    }

    #[test]
    fn any_other_argument_is_unknown() {
        assert_eq!(parse(&["build"]), Command::Unknown("build".to_owned()));
    }
}
