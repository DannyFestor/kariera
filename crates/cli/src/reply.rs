//! The only place that writes to the terminal.

use std::process::ExitCode;

use crate::command::Command;

const USAGE: &str = "Usage: kariera [--help | --version]

Builds career documents from an Obsidian vault. No commands exist yet.";

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
            Command::Unknown(argument) => Self::usage_error(argument),
        }
    }

    pub(crate) fn send(self) -> ExitCode {
        match self.stream {
            Stream::Stdout => write_stdout(&self.text),
            Stream::Stderr => write_stderr(&self.text),
        }
        self.exit_code
    }

    const fn success(text: String) -> Self {
        Self {
            stream: Stream::Stdout,
            text,
            exit_code: ExitCode::SUCCESS,
        }
    }

    fn usage_error(argument: &str) -> Self {
        Self {
            stream: Stream::Stderr,
            text: format!("kariera: unknown argument {argument:?}\n\n{USAGE}"),
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
    fn an_unknown_argument_is_reported_with_the_usage_and_fails() {
        let reply = Reply::for_command(&Command::Unknown("build".to_owned()));

        assert_eq!(reply.stream, Stream::Stderr);
        assert!(reply.text.contains("\"build\""), "{}", reply.text);
        assert!(reply.text.ends_with(USAGE), "{}", reply.text);
        assert_eq!(reply.exit_code, ExitCode::from(USAGE_ERROR));
    }
}
