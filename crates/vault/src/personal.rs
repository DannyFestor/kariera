//! `~/.config/kariera/personal.yaml`: the owner's contact and personal details, which never live in the vault
//! (format in the README).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use kariera_domain::{Personal, Photo, PublicDetails, Text};
use thiserror::Error;

use crate::cursor::Cursor;
use crate::fields::read_map;
use crate::names::Names;
use crate::value::FromNode;
use crate::yaml::{self, Entry, Node, Value};
use crate::{Location, Problem};

#[derive(Debug, Error)]
pub enum PersonalError {
    #[error("no home directory is known, so there is no personal.yaml path")]
    NoHome,
    #[error("cannot read {}: {source}", path.display())]
    Unreadable { path: PathBuf, source: io::Error },
    #[error("{}:{line}: {message}", path.display())]
    Invalid {
        path: PathBuf,
        line: usize,
        message: String,
    },
}

/// The file as written, before the photo it names is read.
#[derive(Debug, Default)]
struct PersonalFile {
    personal: Personal,
    photo: Option<PathBuf>,
}

/// A problem with one line of the file.
struct Invalid {
    line: usize,
    message: String,
}

/// Beside `config.toml`.
pub fn default_personal_path() -> Result<PathBuf, PersonalError> {
    home().map(|home| home.join(".config/kariera/personal.yaml"))
}

/// A missing file holds no details: every field is optional, and so is the file.
pub fn load_personal(path: &Path) -> Result<Personal, PersonalError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Personal::default()),
        Err(source) => return Err(unreadable(path, source)),
    };
    let file = PersonalFile::parse(&text, path, home)?;
    let photo = file
        .photo
        .map(|photo| {
            fs::read(&photo)
                .map(Photo)
                .map_err(|source| unreadable(&photo, source))
        })
        .transpose()?;
    Ok(Personal {
        photo,
        ..file.personal
    })
}

fn home() -> Result<PathBuf, PersonalError> {
    std::env::home_dir().ok_or(PersonalError::NoHome)
}

fn unreadable(path: &Path, source: io::Error) -> PersonalError {
    PersonalError::Unreadable {
        path: path.to_owned(),
        source,
    }
}

impl PersonalFile {
    /// `home` is asked only when `photo` starts with `~/`.
    fn parse(
        text: &str,
        path: &Path,
        home: impl FnOnce() -> Result<PathBuf, PersonalError>,
    ) -> Result<Self, PersonalError> {
        let invalid = |Invalid { line, message }| PersonalError::Invalid {
            path: path.to_owned(),
            line,
            message,
        };
        let root = yaml::parse(text, 1).map_err(|error| {
            invalid(Invalid {
                line: error.line,
                message: error.message,
            })
        })?;
        let Some(root) = root else {
            return Ok(Self::default());
        };
        let Value::Mapping(entries) = &root.value else {
            return Err(invalid(Invalid {
                line: root.line,
                message: "must be a map of the keys in the README".to_owned(),
            }));
        };
        let mut file = Self::default();
        let mut photo = None;
        for entry in entries {
            file.read(entry, &mut photo).map_err(invalid)?;
        }
        file.photo = photo
            .map(|written: Text| photo_path(written.as_str(), path, home))
            .transpose()?;
        Ok(file)
    }

    fn read(&mut self, entry: &Entry, photo: &mut Option<Text>) -> Result<(), Invalid> {
        let personal = &mut self.personal;
        match entry.key.as_str() {
            "email" => personal.email = value(entry)?,
            "phone" => personal.phone = value(entry)?,
            "location" => personal.location = value(entry)?,
            "birth_date" => personal.birth_date = value(entry)?,
            "nationality" => personal.nationality = value(entry)?,
            "photo" => *photo = value(entry)?,
            "name_kana" => personal.name_kana = value(entry)?,
            "gender" => personal.gender = value(entry)?,
            "postal_code" => personal.postal_code = value(entry)?,
            "address" => personal.address = value(entry)?,
            "address_kana" => personal.address_kana = value(entry)?,
            "residence_status" => personal.residence_status = value(entry)?,
            "work_restriction" => personal.work_restriction = value(entry)?,
            "residence_expiry" => personal.residence_expiry = value(entry)?,
            "residence_note" => personal.residence_note = value(entry)?,
            "public" => personal.public = value(entry)?.unwrap_or_default(),
            other => {
                return Err(Invalid {
                    line: entry.value.line,
                    message: format!("unknown key `{other}`"),
                });
            }
        }
        Ok(())
    }
}

/// No key but these two, so nothing else can reach a public document.
impl FromNode for PublicDetails {
    fn from_node(node: &Node, cursor: &Cursor<'_>) -> Result<Self, Vec<Problem>> {
        read_map(node, cursor, |fields| {
            let email = fields.optional("email");
            let location = fields.optional("location");
            Ok(Self {
                email: email?,
                location: location?,
            })
        })
    }
}

/// Read as a note field of the same type is, with no notes to link to.
fn value<T: FromNode>(entry: &Entry) -> Result<Option<T>, Invalid> {
    if T::is_empty(&entry.value) {
        return Ok(None);
    }
    let names = Names::default();
    let cursor = Cursor::root(&names).key(&entry.key);
    T::from_node(&entry.value, &cursor)
        .map(Some)
        .map_err(|problems| first_invalid(problems, entry))
}

fn first_invalid(problems: Vec<Problem>, entry: &Entry) -> Invalid {
    let problem = problems.into_iter().next();
    let line = problem
        .as_ref()
        .and_then(|problem| problem.line)
        .unwrap_or(entry.value.line);
    let message = problem.map_or_else(
        || format!("`{}` cannot be read", entry.key),
        |problem| match problem.location {
            Location::Field(path) => format!("`{path}` {}", problem.message),
            Location::Note | Location::Frontmatter | Location::Section(_) => problem.message,
        },
    );
    Invalid { line, message }
}

/// Relative to the folder of `personal.yaml`, or to the home directory when it starts with `~/`; an absolute path
/// stays as it is.
fn photo_path(
    written: &str,
    personal_path: &Path,
    home: impl FnOnce() -> Result<PathBuf, PersonalError>,
) -> Result<PathBuf, PersonalError> {
    if let Some(in_home) = written.strip_prefix("~/") {
        return home().map(|home| home.join(in_home));
    }
    let folder = personal_path.parent().unwrap_or_else(|| Path::new(""));
    Ok(folder.join(written))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use kariera_domain::{Personal, PublicDetails, Text, Translated};

    use super::{PersonalError, PersonalFile};

    const PATH: &str = "/home/robin/.config/kariera/personal.yaml";

    fn text(value: &str) -> Text {
        value.parse().unwrap()
    }

    fn parse(text: &str) -> Result<PersonalFile, PersonalError> {
        PersonalFile::parse(text, Path::new(PATH), || Ok(PathBuf::from("/home/robin")))
    }

    #[test]
    fn every_field_is_read() {
        let file = parse(
            "email: robin@example.com\nphone: +81 90 0000 0000\nlocation: Sapporo, Japan\n\
             birth_date: 1990-01-01\nnationality:\n  en: German\n  ja: ドイツ\n  de: deutsch\n\
             photo: photo.jpg\nname_kana: ロビン　ベイシュピール\ngender: 男\npostal_code: 001-0000\n\
             address: 北海道札幌市北区北0条西0丁目\naddress_kana: ほっかいどう さっぽろし\n\
             residence_status: 技術・人文知識・国際業務\nwork_restriction: 就労制限なし\n\
             residence_expiry: 2028-03-31\nresidence_note: 更新申請中\n",
        )
        .unwrap();

        assert_eq!(
            file.personal,
            Personal {
                email: Some(text("robin@example.com")),
                phone: Some(text("+81 90 0000 0000")),
                location: Some(text("Sapporo, Japan")),
                birth_date: Some("1990-01-01".parse().unwrap()),
                nationality: Some(Translated {
                    en: text("German"),
                    ja: Some(text("ドイツ")),
                    de: Some(text("deutsch")),
                }),
                photo: None,
                name_kana: Some(text("ロビン　ベイシュピール")),
                gender: Some(text("男")),
                postal_code: Some(text("001-0000")),
                address: Some(text("北海道札幌市北区北0条西0丁目")),
                address_kana: Some(text("ほっかいどう さっぽろし")),
                residence_status: Some(text("技術・人文知識・国際業務")),
                work_restriction: Some(text("就労制限なし")),
                residence_expiry: Some("2028-03-31".parse().unwrap()),
                residence_note: Some(text("更新申請中")),
                public: PublicDetails::default(),
            }
        );
        assert_eq!(
            file.photo,
            Some(PathBuf::from("/home/robin/.config/kariera/photo.jpg"))
        );
    }

    #[test]
    fn a_nationality_is_a_map_of_languages_with_english_required() {
        let single = parse("nationality: deutsch\n").unwrap_err();
        let without_english = parse("nationality:\n  ja: ドイツ\n").unwrap_err();

        assert_eq!(
            single.to_string(),
            format!("{PATH}:1: `nationality` must be a map of fields")
        );
        assert_eq!(
            without_english.to_string(),
            format!("{PATH}:2: `nationality.en` is required")
        );
    }

    #[test]
    fn every_field_is_optional() {
        let file = parse("email:\n").unwrap();

        assert_eq!(file.personal, Personal::default());
        assert_eq!(file.photo, None);
        assert_eq!(parse("").unwrap().personal, Personal::default());
    }

    #[test]
    fn the_public_map_is_read_apart_from_the_private_details() {
        let file = parse(
            "email: robin@example.com\npublic:\n  email: hello@example.com\n  location: Japan\n",
        )
        .unwrap();

        assert_eq!(
            file.personal,
            Personal {
                email: Some(text("robin@example.com")),
                public: PublicDetails {
                    email: Some(text("hello@example.com")),
                    location: Some(text("Japan")),
                },
                ..Personal::default()
            }
        );
    }

    #[test]
    fn a_public_map_holds_only_the_email_and_the_location() {
        let error =
            parse("public:\n  email: hello@example.com\n  phone: +81 90 0000 0000\n").unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("{PATH}:3: `public.phone` is not a field the schema allows here")
        );
    }

    #[test]
    fn a_photo_in_the_home_directory_or_at_an_absolute_path_is_found_there() {
        assert_eq!(
            parse("photo: ~/Pictures/me.png\n").unwrap().photo,
            Some(PathBuf::from("/home/robin/Pictures/me.png"))
        );
        assert_eq!(
            parse("photo: /srv/me.png\n").unwrap().photo,
            Some(PathBuf::from("/srv/me.png"))
        );
    }

    #[test]
    fn an_unknown_key_is_refused_with_its_line() {
        let error = parse("email: robin@example.com\nmail: robin@example.com\n").unwrap_err();

        assert_eq!(error.to_string(), format!("{PATH}:2: unknown key `mail`"));
    }

    #[test]
    fn a_malformed_value_is_refused_with_its_line_and_key() {
        let error = parse("birth_date: 01.01.1990\n").unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("{PATH}:1: `birth_date` must be a date written YYYY-MM-DD, such as 2026-03-04")
        );
    }

    #[test]
    fn a_file_that_is_not_a_map_is_refused() {
        let error = parse("- robin@example.com\n").unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("{PATH}:1: must be a map of the keys in the README")
        );
    }
}
