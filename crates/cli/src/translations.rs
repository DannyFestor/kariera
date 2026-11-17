//! `kariera translations`: which translations are out of date, and recording that one is up to date (schema §1.6.1).

use kariera_domain::{Fingerprint, TranslationLanguage};
use kariera_vault::Translation;

use crate::reply::Reply;
use crate::vault_access::open_vault;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TranslationsCommand {
    Stale,
    Mark {
        note: String,
        language: TranslationLanguage,
    },
}

/// The options as given, before it is known that they make a command.
#[derive(Debug, Default)]
struct Options {
    stale: bool,
    note: Option<String>,
    language: Option<TranslationLanguage>,
}

impl TranslationsCommand {
    /// The error is what to tell the user about the arguments.
    pub(crate) fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut arguments = arguments.into_iter();
        let mut options = Options::default();
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--stale" => options.stale = true,
                "--mark" => options.note = Some(value_of("--mark", "a note name", &mut arguments)?),
                "--lang" => options.language = Some(language_of(&mut arguments)?),
                _ => return Err(format!("unknown argument {argument:?}")),
            }
        }
        options.into_command()
    }

    pub(crate) fn run(&self) -> Reply {
        match self.report() {
            Ok(text) => Reply::success(text),
            Err(message) => Reply::failure(message),
        }
    }

    fn report(&self) -> Result<String, String> {
        let vault = open_vault()?;
        match self {
            Self::Stale => Ok(stale_report(&vault.translations())),
            Self::Mark { note, language } => vault
                .mark(note, *language)
                .map(|fingerprint| marked_report(note, *language, &fingerprint))
                .map_err(|error| error.to_string()),
        }
    }
}

impl Options {
    fn into_command(self) -> Result<TranslationsCommand, String> {
        match (self.stale, self.note, self.language) {
            (true, None, None) => Ok(TranslationsCommand::Stale),
            (false, Some(note), Some(language)) => Ok(TranslationsCommand::Mark { note, language }),
            (true, _, _) => Err("--stale cannot be combined with --mark or --lang".to_owned()),
            (false, Some(_), None) => Err("--mark needs --lang <ja|de>".to_owned()),
            (false, None, Some(_)) => Err("--lang needs --mark <note>".to_owned()),
            (false, None, None) => {
                Err("translations needs --stale, or --mark <note> --lang <ja|de>".to_owned())
            }
        }
    }
}

fn value_of(
    option: &str,
    what: &str,
    arguments: &mut impl Iterator<Item = String>,
) -> Result<String, String> {
    arguments
        .next()
        .ok_or_else(|| format!("{option} needs {what}"))
}

fn language_of(
    arguments: &mut impl Iterator<Item = String>,
) -> Result<TranslationLanguage, String> {
    value_of("--lang", "ja or de", arguments)?
        .parse()
        .map_err(|error| format!("--lang {error}"))
}

fn stale_report(translations: &[Translation]) -> String {
    let lines: Vec<String> = translations
        .iter()
        .filter(|translation| translation.is_stale())
        .map(stale_line)
        .collect();
    if lines.is_empty() {
        "No translation is stale.".to_owned()
    } else {
        lines.join("\n")
    }
}

/// Tab separated, so a script can split it. `empty` is never translated, `differs` is English changed since.
fn stale_line(translation: &Translation) -> String {
    let reason = if translation.stored.is_none() {
        "empty"
    } else {
        "differs"
    };
    format!(
        "{}\t{}\t{reason}",
        translation.name,
        translation.language.code()
    )
}

fn marked_report(note: &str, language: TranslationLanguage, fingerprint: &Fingerprint) -> String {
    format!(
        "Marked {note}: {} is translated from {fingerprint}",
        language.code()
    )
}

#[cfg(test)]
mod tests {
    use kariera_domain::{Fingerprint, TranslationLanguage};
    use kariera_vault::Translation;

    use super::{TranslationsCommand, marked_report, stale_report};

    fn parse(arguments: &[&str]) -> Result<TranslationsCommand, String> {
        TranslationsCommand::parse(arguments.iter().map(ToString::to_string))
    }

    fn mark(note: &str, language: TranslationLanguage) -> TranslationsCommand {
        TranslationsCommand::Mark {
            note: note.to_owned(),
            language,
        }
    }

    fn translation(
        name: &str,
        language: TranslationLanguage,
        stored: Option<&str>,
        current: &str,
    ) -> Translation {
        Translation {
            name: name.parse().unwrap(),
            path: format!("Skills/{name}.md").into(),
            language,
            stored: stored.map(|stored| stored.parse::<Fingerprint>().unwrap()),
            current: current.parse().unwrap(),
        }
    }

    #[test]
    fn stale_asks_for_the_stale_report() {
        assert_eq!(parse(&["--stale"]), Ok(TranslationsCommand::Stale));
    }

    #[test]
    fn mark_needs_a_note_and_a_language_in_any_order() {
        let expected = Ok(mark("Acme - Shop", TranslationLanguage::German));

        assert_eq!(parse(&["--mark", "Acme - Shop", "--lang", "de"]), expected);
        assert_eq!(parse(&["--lang", "de", "--mark", "Acme - Shop"]), expected);
    }

    #[test]
    fn a_language_must_be_ja_or_de() {
        assert_eq!(
            parse(&["--mark", "Go", "--lang", "en"]),
            Err("--lang must be ja or de".to_owned())
        );
    }

    #[test]
    fn an_option_without_its_value_is_refused() {
        assert_eq!(
            parse(&["--mark"]),
            Err("--mark needs a note name".to_owned())
        );
        assert_eq!(
            parse(&["--mark", "Go", "--lang"]),
            Err("--lang needs ja or de".to_owned())
        );
    }

    #[test]
    fn options_that_do_not_make_a_command_are_refused() {
        let cases: [(&[&str], &str); 5] = [
            (
                &[],
                "translations needs --stale, or --mark <note> --lang <ja|de>",
            ),
            (&["--mark", "Go"], "--mark needs --lang <ja|de>"),
            (&["--lang", "ja"], "--lang needs --mark <note>"),
            (
                &["--stale", "--lang", "ja"],
                "--stale cannot be combined with --mark or --lang",
            ),
            (
                &["--stale", "--mark", "Go", "--lang", "ja"],
                "--stale cannot be combined with --mark or --lang",
            ),
        ];
        for (arguments, message) in cases {
            assert_eq!(parse(arguments), Err(message.to_owned()), "{arguments:?}");
        }
    }

    #[test]
    fn an_unknown_option_is_refused() {
        assert_eq!(
            parse(&["--all"]),
            Err("unknown argument \"--all\"".to_owned())
        );
    }

    #[test]
    fn only_stale_translations_are_listed_with_their_reason() {
        let translations = [
            translation("Go", TranslationLanguage::Japanese, None, "0123456789ab"),
            translation(
                "Go",
                TranslationLanguage::German,
                Some("0123456789ab"),
                "0123456789ab",
            ),
            translation(
                "Rust",
                TranslationLanguage::German,
                Some("ffffffffffff"),
                "0123456789ab",
            ),
        ];

        assert_eq!(
            stale_report(&translations),
            "Go\tja\tempty\nRust\tde\tdiffers"
        );
    }

    #[test]
    fn nothing_stale_is_said_in_words() {
        let translations = [translation(
            "Go",
            TranslationLanguage::German,
            Some("0123456789ab"),
            "0123456789ab",
        )];

        assert_eq!(stale_report(&translations), "No translation is stale.");
        assert_eq!(stale_report(&[]), "No translation is stale.");
    }

    #[test]
    fn marking_names_the_note_the_language_and_the_fingerprint() {
        let fingerprint = "0123456789ab".parse().unwrap();

        assert_eq!(
            marked_report("Go", TranslationLanguage::Japanese, &fingerprint),
            "Marked Go: ja is translated from 0123456789ab"
        );
    }
}
