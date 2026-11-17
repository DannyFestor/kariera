//! `kariera validate`: checks the configured vault and reports what is wrong with it.

use std::process::ExitCode;

use kariera_validate::{Finding, Report};
use kariera_vault::Location;

use crate::clock::current_month;
use crate::json::Json;
use crate::reply::Reply;
use crate::vault_access::vault_root;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Format {
    Human,
    Json,
}

impl Format {
    /// The arguments after `validate`. The error is what to tell the user about them.
    pub(crate) fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut format = Self::Human;
        for argument in arguments {
            match argument.as_str() {
                "--json" => format = Self::Json,
                _ => return Err(format!("unknown argument {argument:?}")),
            }
        }
        Ok(format)
    }
}

pub(crate) fn reply(format: Format) -> Reply {
    match run(format) {
        Ok(reply) => reply,
        Err(message) => Reply::failure(message),
    }
}

fn run(format: Format) -> Result<Reply, String> {
    let report = kariera_validate::validate(&vault_root()?, current_month()?)
        .map_err(|error| error.to_string())?;
    let text = match format {
        Format::Human => human(&report),
        Format::Json => json(&report).render(),
    };
    Ok(if report.has_errors() {
        Reply::output(text, ExitCode::FAILURE)
    } else {
        Reply::success(text)
    })
}

fn human(report: &Report) -> String {
    let mut lines: Vec<String> = report.findings().iter().map(ToString::to_string).collect();
    if !report.is_complete() {
        lines.push(
            "Some notes could not be read, so the rules that need every note (a Contract listed by exactly \
             one Engagement, exactly one Profile) did not run: fix the problems above first."
                .to_owned(),
        );
    }
    lines.push(summary(report));
    lines.join("\n")
}

fn summary(report: &Report) -> String {
    let notes = format!(" in {} notes", report.notes_checked());
    if report.findings().is_empty() {
        return format!("No problems{notes}.");
    }
    format!(
        "{}, {}{notes}.",
        counted(report.errors(), "error"),
        counted(report.warnings(), "warning")
    )
}

fn counted(count: usize, noun: &str) -> String {
    let plural = if count == 1 { "" } else { "s" };
    format!("{count} {noun}{plural}")
}

fn json(report: &Report) -> Json {
    Json::Object(vec![
        ("errors", Json::Number(report.errors())),
        ("warnings", Json::Number(report.warnings())),
        ("notes_checked", Json::Number(report.notes_checked())),
        ("complete", Json::Bool(report.is_complete())),
        (
            "findings",
            Json::Array(report.findings().iter().map(finding_json).collect()),
        ),
    ])
}

fn finding_json(finding: &Finding) -> Json {
    Json::Object(vec![
        ("path", Json::string(finding.path.display().to_string())),
        ("line", finding.line.map_or(Json::Null, Json::Number)),
        ("severity", Json::string(finding.severity.name())),
        ("rule", Json::string(finding.rule.code())),
        ("location", location_json(&finding.location)),
        ("message", Json::string(&finding.message)),
    ])
}

fn location_json(location: &Location) -> Json {
    let (kind, name) = match location {
        Location::Note => ("note", None),
        Location::Frontmatter => ("frontmatter", None),
        Location::Field(path) => ("field", Some(path)),
        Location::Section(heading) => ("section", Some(heading)),
    };
    Json::Object(vec![
        ("kind", Json::string(kind)),
        ("name", name.map_or(Json::Null, Json::string)),
    ])
}

#[cfg(test)]
mod tests {
    use super::{Format, counted};

    fn parse(arguments: &[&str]) -> Result<Format, String> {
        Format::parse(arguments.iter().map(ToString::to_string))
    }

    #[test]
    fn without_flags_the_report_is_for_people() {
        assert_eq!(parse(&[]), Ok(Format::Human));
    }

    #[test]
    fn json_is_for_programs() {
        assert_eq!(parse(&["--json"]), Ok(Format::Json));
    }

    #[test]
    fn an_unknown_argument_is_the_error() {
        assert_eq!(
            parse(&["--json", "--yaml"]),
            Err("unknown argument \"--yaml\"".to_owned())
        );
        assert_eq!(
            parse(&["vault"]),
            Err("unknown argument \"vault\"".to_owned())
        );
    }

    #[test]
    fn a_count_is_plural_except_for_one() {
        assert_eq!(counted(0, "error"), "0 errors");
        assert_eq!(counted(1, "error"), "1 error");
        assert_eq!(counted(2, "warning"), "2 warnings");
    }
}
