//! The only place that writes to the terminal.

use std::process::ExitCode;

use crate::command::Command;
use crate::new_note;
use crate::validate;

const USAGE: &str = "Usage: kariera [--help | --version]
       kariera new log --engagement <name> --month <YYYY-MM>
       kariera new project --engagement <name> --title <title>
       kariera new engagement (--client <name> | --employer <name>) --type <type> --start <YYYY-MM>
       kariera validate [--json]
       kariera build [--doc <cv|shokumu-keirekisho|skill-sheet>] [--lang <en|ja|de>]
       kariera translations --stale
       kariera translations --mark <note> --lang <ja|de>

Builds career documents from an Obsidian vault.

new                    Creates a note from the vault's Templates, with what the arguments say filled in
validate               Checks every note against the schema and the other notes; exits 1 on errors, --json is for programs
build                  Writes every document in each of its languages as PDF into the vault's Generated/
translations --stale   Lists the translations whose English text changed since they were marked
translations --mark    Records that a note's translation matches its current English text";

// The exit code most Unix tools use for a command line they do not understand.
const USAGE_ERROR: u8 = 2;

#[derive(Debug, PartialEq, Eq)]
enum Stream {
    Stdout,
    Stderr,
}

#[derive(Debug)]
pub(crate) struct Reply {
    stream: Stream,
    text: String,
    exit_code: ExitCode,
}

impl Reply {
    pub(crate) fn for_command(command: &Command) -> Self {
        match command {
            Command::Help => Self::success(USAGE.to_owned()),
            Command::Version => Self::success(version()),
            Command::Build(build) => build.run(),
            Command::New(note) => new_note::reply(note),
            Command::Translations(translations) => translations.run(),
            Command::Validate(format) => validate::reply(*format),
            Command::Unknown(argument) => {
                Self::usage_error(&format!("unknown argument {argument:?}"))
            }
            Command::Misused(message) => Self::usage_error(message),
        }
    }

    pub(crate) fn send(self) -> ExitCode {
        match self.stream {
            Stream::Stdout => write_stdout(&self.text),
            Stream::Stderr => write_stderr(&self.text),
        }
        self.exit_code
    }

    pub(crate) const fn success(text: String) -> Self {
        Self {
            stream: Stream::Stdout,
            text,
            exit_code: ExitCode::SUCCESS,
        }
    }

    /// The result of a command that ran and has something to say on stdout, with the exit code it decides.
    pub(crate) const fn output(text: String, exit_code: ExitCode) -> Self {
        Self {
            stream: Stream::Stdout,
            text,
            exit_code,
        }
    }

    pub(crate) const fn failure(text: String) -> Self {
        Self {
            stream: Stream::Stderr,
            text,
            exit_code: ExitCode::FAILURE,
        }
    }

    fn usage_error(message: &str) -> Self {
        Self {
            stream: Stream::Stderr,
            text: format!("kariera: {message}\n\n{USAGE}"),
            exit_code: ExitCode::from(USAGE_ERROR),
        }
    }
}

fn version() -> String {
    format!("kariera {}", env!("CARGO_PKG_VERSION"))
}

#[expect(
    clippy::print_stdout,
    reason = "the cli is the only crate that prints (ADR 0001)"
)]
fn write_stdout(text: &str) {
    println!("{text}");
}

#[expect(
    clippy::print_stderr,
    reason = "the cli is the only crate that prints (ADR 0001)"
)]
fn write_stderr(text: &str) {
    eprintln!("{text}");
}

#[cfg(test)]
mod tests {
    use std::process::ExitCode;

    use super::{Reply, Stream, USAGE, USAGE_ERROR};
    use crate::command::Command;

    #[test]
    fn help_prints_the_usage_and_succeeds() {
        let reply = Reply::for_command(&Command::Help);

        assert_eq!(reply.stream, Stream::Stdout);
        assert_eq!(reply.text, USAGE);
        assert_eq!(reply.exit_code, ExitCode::SUCCESS);
    }

    #[test]
    fn version_prints_the_package_version_and_succeeds() {
        let reply = Reply::for_command(&Command::Version);

        assert_eq!(reply.stream, Stream::Stdout);
        assert_eq!(reply.text, format!("kariera {}", env!("CARGO_PKG_VERSION")));
        assert_eq!(reply.exit_code, ExitCode::SUCCESS);
    }

    #[test]
    fn a_failure_goes_to_stderr_as_it_is_and_fails() {
        let reply = Reply::failure("cannot read the vault".to_owned());

        assert_eq!(reply.stream, Stream::Stderr);
        assert_eq!(reply.text, "cannot read the vault");
        assert_eq!(reply.exit_code, ExitCode::FAILURE);
    }

    #[test]
    fn a_misused_command_is_reported_with_the_usage_and_fails() {
        let reply = Reply::for_command(&Command::Misused("--mark needs --lang <ja|de>".to_owned()));

        assert_eq!(reply.stream, Stream::Stderr);
        assert!(
            reply
                .text
                .starts_with("kariera: --mark needs --lang <ja|de>"),
            "{}",
            reply.text
        );
        assert!(reply.text.ends_with(USAGE), "{}", reply.text);
        assert_eq!(reply.exit_code, ExitCode::from(USAGE_ERROR));
    }

    #[test]
    fn an_unknown_argument_is_reported_with_the_usage_and_fails() {
        let reply = Reply::for_command(&Command::Unknown("publish".to_owned()));

        assert_eq!(reply.stream, Stream::Stderr);
        assert!(reply.text.contains("\"publish\""), "{}", reply.text);
        assert!(reply.text.ends_with(USAGE), "{}", reply.text);
        assert_eq!(reply.exit_code, ExitCode::from(USAGE_ERROR));
    }
}
