//! `kariera new log|project|engagement`: reads the arguments into a request and has the vault create the note.

mod flags;

use std::path::PathBuf;

use kariera_domain::EngagementType;
use kariera_vault::{NewEngagement, NewLog, NewNote, NewProject, OrganizationRole};

use crate::reply::Reply;
use crate::vault_access::open_vault;
use flags::{ArgumentError, Flags};

pub(crate) fn parse(arguments: impl IntoIterator<Item = String>) -> Result<NewNote, ArgumentError> {
    let mut arguments = arguments.into_iter();
    let note_type = arguments.next().ok_or(ArgumentError::MissingNoteType)?;
    let mut flags = Flags::parse(arguments)?;
    let note = match note_type.as_str() {
        "log" => parse_log(&mut flags)?,
        "project" => parse_project(&mut flags)?,
        "engagement" => parse_engagement(&mut flags)?,
        _ => return Err(ArgumentError::UnknownNoteType(note_type)),
    };
    flags.finish()?;
    Ok(note)
}

pub(crate) fn reply(note: &NewNote) -> Reply {
    match create(note) {
        Ok(path) => Reply::success(format!("Created {}", path.display())),
        Err(message) => Reply::failure(message),
    }
}

fn parse_log(flags: &mut Flags) -> Result<NewNote, ArgumentError> {
    Ok(NewNote::Log(NewLog {
        engagement: flags.required("engagement")?,
        month: flags.required("month")?,
    }))
}

fn parse_project(flags: &mut Flags) -> Result<NewNote, ArgumentError> {
    Ok(NewNote::Project(NewProject {
        engagement: flags.required("engagement")?,
        title: flags.required("title")?,
    }))
}

fn parse_engagement(flags: &mut Flags) -> Result<NewNote, ArgumentError> {
    let (role, organization) = match (flags.optional("client")?, flags.optional("employer")?) {
        (Some(client), None) => (OrganizationRole::Client, client),
        (None, Some(employer)) => (OrganizationRole::Employer, employer),
        _ => return Err(ArgumentError::ClientXorEmployer),
    };
    Ok(NewNote::Engagement(NewEngagement {
        organization,
        role,
        engagement_type: flags.required_slug::<EngagementType>("type")?,
        start: flags.required("start")?,
    }))
}

fn create(note: &NewNote) -> Result<PathBuf, String> {
    open_vault()?
        .create(note)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use kariera_domain::{EngagementType, Month, NoteName, Text};
    use kariera_vault::{NewEngagement, NewLog, NewNote, NewProject, OrganizationRole};

    use super::parse;

    fn parsed(arguments: &[&str]) -> Result<NewNote, String> {
        parse(arguments.iter().map(ToString::to_string)).map_err(|error| error.to_string())
    }

    fn name(text: &str) -> NoteName {
        text.parse().unwrap()
    }

    fn month(text: &str) -> Month {
        text.parse().unwrap()
    }

    #[test]
    fn a_log_takes_engagement_and_month() {
        let note = parsed(&["log", "--engagement", "2025-06 Acme", "--month", "2026-11"]);

        assert_eq!(
            note,
            Ok(NewNote::Log(NewLog {
                engagement: name("2025-06 Acme"),
                month: month("2026-11"),
            }))
        );
    }

    #[test]
    fn a_project_takes_engagement_and_title() {
        let note = parsed(&[
            "project",
            "--title",
            "Shop Backend",
            "--engagement=2025-06 Acme",
        ]);

        assert_eq!(
            note,
            Ok(NewNote::Project(NewProject {
                engagement: name("2025-06 Acme"),
                title: "Shop Backend".parse::<Text>().unwrap(),
            }))
        );
    }

    #[test]
    fn an_engagement_takes_a_client_or_an_employer_with_type_and_start() {
        let client = parsed(&[
            "engagement",
            "--client",
            "Acme",
            "--type",
            "freelance",
            "--start",
            "2026-01",
        ]);
        let employer = parsed(&[
            "engagement",
            "--employer",
            "Initech",
            "--type",
            "employment",
            "--start",
            "2026-01",
        ]);

        assert_eq!(
            client,
            Ok(NewNote::Engagement(NewEngagement {
                organization: name("Acme"),
                role: OrganizationRole::Client,
                engagement_type: EngagementType::Freelance,
                start: month("2026-01"),
            }))
        );
        assert_eq!(
            employer,
            Ok(NewNote::Engagement(NewEngagement {
                organization: name("Initech"),
                role: OrganizationRole::Employer,
                engagement_type: EngagementType::Employment,
                start: month("2026-01"),
            }))
        );
    }

    #[test]
    fn exactly_one_of_client_and_employer_is_needed() {
        let both = [
            "engagement",
            "--client",
            "A",
            "--employer",
            "B",
            "--type",
            "freelance",
            "--start",
            "2026-01",
        ];
        let neither = ["engagement", "--type", "freelance", "--start", "2026-01"];
        let message = "an engagement needs exactly one of --client and --employer";

        assert_eq!(parsed(&both), Err(message.to_owned()));
        assert_eq!(parsed(&neither), Err(message.to_owned()));
    }

    #[test]
    fn a_missing_note_type_or_an_unknown_one_is_refused() {
        assert_eq!(
            parsed(&[]),
            Err("expected log, project or engagement".to_owned())
        );
        assert_eq!(
            parsed(&["skill"]),
            Err("\"skill\" is not log, project or engagement".to_owned())
        );
    }

    #[test]
    fn a_missing_flag_is_named() {
        assert_eq!(
            parsed(&["log", "--engagement", "2025-06 Acme"]),
            Err("--month is required".to_owned())
        );
    }

    #[test]
    fn a_flag_that_does_not_belong_to_the_note_type_is_refused() {
        let note = parsed(&[
            "log",
            "--engagement",
            "A",
            "--month",
            "2026-11",
            "--title",
            "T",
        ]);

        assert_eq!(note, Err("unknown flag --title".to_owned()));
    }

    #[test]
    fn an_invalid_value_names_flag_and_reason() {
        assert_eq!(
            parsed(&["log", "--engagement", "A", "--month", "2026-13"]),
            Err("--month \"2026-13\": must be a month written YYYY-MM, such as 2024-05".to_owned())
        );
    }

    #[test]
    fn an_unknown_engagement_type_lists_the_allowed_ones() {
        let note = parsed(&[
            "engagement",
            "--client",
            "A",
            "--type",
            "gig",
            "--start",
            "2026-01",
        ]);

        assert_eq!(
            note,
            Err("--type \"gig\": expected one of employment, dispatch, internship, freelance, own-business".to_owned())
        );
    }
}
