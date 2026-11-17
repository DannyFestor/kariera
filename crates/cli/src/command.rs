#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    Help,
    Version,
    Unknown(String),
}

impl Command {
    pub(crate) fn parse(arguments: impl IntoIterator<Item = String>) -> Self {
        arguments
            .into_iter()
            .next()
            .map_or(Self::Help, Self::from_argument)
    }

    fn from_argument(argument: String) -> Self {
        match argument.as_str() {
            "help" | "--help" | "-h" => Self::Help,
            "--version" | "-V" => Self::Version,
            _ => Self::Unknown(argument),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Command;

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
    fn any_other_argument_is_unknown() {
        assert_eq!(parse(&["build"]), Command::Unknown("build".to_owned()));
    }
}
