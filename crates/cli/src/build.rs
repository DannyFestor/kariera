//! `kariera build`: writes every document in every language it is written in as PDF into the vault's `Generated/`,
//! the CV and the 職務経歴書 also as DOCX, and the Skill Sheet as XLSX, beside the PDF. The 履歴書 is a PDF only.

use std::collections::BTreeSet;
use std::path::PathBuf;

use kariera_documents::{Document, DocumentKind, Edition, Inputs, MissingTranslation, build};
use kariera_domain::Language;
use kariera_render_docx::{render as render_docx, renders as has_docx_layout};
use kariera_render_typst::render;
use kariera_render_xlsx::render as render_xlsx;
use kariera_vault::{Vault, default_personal_path, load_personal};

use crate::clock::today;
use crate::reply::Reply;
use crate::translations::value_of;
use crate::vault_access::open_vault;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BuildCommand {
    editions: Vec<Edition>,
}

impl BuildCommand {
    /// The error is what to tell the user about the arguments.
    pub(crate) fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut arguments = arguments.into_iter();
        let mut kind = None;
        let mut language = None;
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--doc" => kind = Some(kind_of(&mut arguments)?),
                "--lang" => language = Some(language_of(&mut arguments)?),
                _ => return Err(format!("unknown argument {argument:?}")),
            }
        }
        editions(kind, language).map(|editions| Self { editions })
    }

    pub(crate) fn run(&self) -> Reply {
        match self.report() {
            Ok(text) => Reply::success(text),
            Err(message) => Reply::failure(message),
        }
    }

    fn report(&self) -> Result<String, String> {
        let vault = open_vault()?;
        let personal = default_personal_path()
            .and_then(|path| load_personal(&path))
            .map_err(|error| error.to_string())?;
        let inputs = Inputs {
            career: vault.career(),
            personal: &personal,
            today: today()?,
        };
        let mut written = Vec::new();
        let mut missing = BTreeSet::new();
        for &edition in &self.editions {
            let built = build_one(&vault, &inputs, edition)?;
            written.extend(built.lines);
            missing.extend(built.missing_translations);
        }
        written.extend(missing.iter().map(missing_line));
        Ok(written.join("\n"))
    }
}

/// What building one edition did.
struct Written {
    lines: Vec<String>,
    missing_translations: Vec<MissingTranslation>,
}

fn build_one(vault: &Vault, inputs: &Inputs<'_>, edition: Edition) -> Result<Written, String> {
    let built = build(edition, inputs).map_err(|error| error.to_string())?;
    let mut lines = vec![write_pdf(vault, &built.document)?];
    if has_docx_layout(edition.kind) {
        lines.push(write_docx(vault, &built.document)?);
    }
    lines.extend(write_xlsx(vault, &built.document)?);
    Ok(Written {
        lines,
        missing_translations: built.missing_translations,
    })
}

fn write_pdf(vault: &Vault, document: &Document) -> Result<String, String> {
    let pdf = render(document).map_err(|error| error.to_string())?;
    let path = write_generated(vault, document, "pdf", &pdf.bytes)?;
    Ok(format!("Wrote {} ({})", path.display(), pages(pdf.pages)))
}

fn write_docx(vault: &Vault, document: &Document) -> Result<String, String> {
    let docx = render_docx(document).map_err(|error| error.to_string())?;
    let path = write_generated(vault, document, "docx", &docx)?;
    Ok(format!("Wrote {}", path.display()))
}

/// Only the Skill Sheet is an XLSX: agencies edit it.
fn write_xlsx(vault: &Vault, document: &Document) -> Result<Option<String>, String> {
    let Document::SkillSheet(sheet) = document else {
        return Ok(None);
    };
    let xlsx = render_xlsx(sheet).map_err(|error| error.to_string())?;
    let path = write_generated(vault, document, "xlsx", &xlsx)?;
    Ok(Some(format!("Wrote {}", path.display())))
}

fn write_generated(
    vault: &Vault,
    document: &Document,
    extension: &str,
    bytes: &[u8],
) -> Result<PathBuf, String> {
    vault
        .write_generated(&format!("{}.{extension}", document.file_stem()), bytes)
        .map_err(|error| error.to_string())
}

fn pages(count: usize) -> String {
    if count == 1 {
        "1 page".to_owned()
    } else {
        format!("{count} pages")
    }
}

fn missing_line(missing: &MissingTranslation) -> String {
    format!(
        "warning: {} has no {} text, so its English is printed",
        missing.note,
        missing.language.code()
    )
}

fn kind_of(arguments: &mut impl Iterator<Item = String>) -> Result<DocumentKind, String> {
    let slugs = joined(&DocumentKind::ALL.map(DocumentKind::slug), "or");
    let value = value_of("--doc", &slugs, arguments)?;
    DocumentKind::from_slug(&value).ok_or_else(|| format!("--doc must be {slugs}"))
}

fn language_of(arguments: &mut impl Iterator<Item = String>) -> Result<Language, String> {
    let codes = joined(&Language::ALL.map(Language::code), "or");
    let value = value_of("--lang", &codes, arguments)?;
    Language::ALL
        .into_iter()
        .find(|language| language.code() == value)
        .ok_or_else(|| format!("--lang must be {codes}"))
}

fn editions(
    kind: Option<DocumentKind>,
    language: Option<Language>,
) -> Result<Vec<Edition>, String> {
    let editions: Vec<Edition> = Edition::all()
        .into_iter()
        .filter(|edition| kind.is_none_or(|kind| edition.kind == kind))
        .filter(|edition| language.is_none_or(|language| edition.language == language))
        .collect();
    match (editions.is_empty(), kind) {
        (true, Some(kind)) => Err(format!(
            "the {} is written in {} only",
            kind.slug(),
            joined(
                &kind
                    .languages()
                    .iter()
                    .map(|written_in| written_in.code())
                    .collect::<Vec<_>>(),
                "and"
            )
        )),
        (true, None) | (false, _) => Ok(editions),
    }
}

fn joined(items: &[&str], conjunction: &str) -> String {
    match items.split_last() {
        Some((last, [])) => (*last).to_owned(),
        Some((last, rest)) => format!("{} {conjunction} {last}", rest.join(", ")),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use kariera_documents::DocumentKind;
    use kariera_domain::Language;

    use super::BuildCommand;

    fn parse(arguments: &[&str]) -> Result<BuildCommand, String> {
        BuildCommand::parse(arguments.iter().map(ToString::to_string))
    }

    fn editions(arguments: &[&str]) -> Vec<String> {
        parse(arguments)
            .unwrap()
            .editions
            .iter()
            .map(|edition| format!("{}-{}", edition.kind.slug(), edition.language.code()))
            .collect()
    }

    #[test]
    fn without_options_every_document_is_built_in_each_of_its_languages() {
        assert_eq!(
            editions(&[]),
            [
                "cv-en",
                "cv-de",
                "shokumu-keirekisho-ja",
                "rirekisho-ja",
                "skill-sheet-ja"
            ]
        );
    }

    #[test]
    fn doc_narrows_to_one_document() {
        assert_eq!(editions(&["--doc", "cv"]), ["cv-en", "cv-de"]);
    }

    #[test]
    fn lang_narrows_to_one_language() {
        assert_eq!(
            editions(&["--lang", "ja"]),
            ["shokumu-keirekisho-ja", "rirekisho-ja", "skill-sheet-ja"]
        );
        assert_eq!(editions(&["--lang", "de", "--doc", "cv"]), ["cv-de"]);
        assert_eq!(editions(&["--doc", "rirekisho"]), ["rirekisho-ja"]);
    }

    #[test]
    fn a_document_in_a_language_it_is_not_written_in_is_refused() {
        assert_eq!(
            parse(&["--doc", "skill-sheet", "--lang", "en"]).map(|_| ()),
            Err("the skill-sheet is written in ja only".to_owned())
        );
        assert_eq!(
            parse(&["--doc", "cv", "--lang", "ja"]).map(|_| ()),
            Err("the cv is written in en and de only".to_owned())
        );
    }

    #[test]
    fn unknown_values_and_arguments_are_refused() {
        let cases: [(&[&str], &str); 5] = [
            (
                &["--doc", "resume"],
                "--doc must be cv, shokumu-keirekisho, rirekisho or skill-sheet",
            ),
            (&["--lang", "fr"], "--lang must be en, ja or de"),
            (
                &["--doc"],
                "--doc needs cv, shokumu-keirekisho, rirekisho or skill-sheet",
            ),
            (&["--lang"], "--lang needs en, ja or de"),
            (&["--pdf"], "unknown argument \"--pdf\""),
        ];
        for (arguments, message) in cases {
            assert_eq!(
                parse(arguments).map(|_| ()),
                Err(message.to_owned()),
                "{arguments:?}"
            );
        }
    }

    #[test]
    fn kinds_and_languages_are_parsed_from_their_codes() {
        assert!(parse(&["--doc", "shokumu-keirekisho"]).is_ok());
        assert_eq!(
            DocumentKind::Cv.languages(),
            [Language::English, Language::German]
        );
    }
}
