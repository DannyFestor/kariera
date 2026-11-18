//! `~/.config/kariera/personal.yaml`: the owner's contact and personal details, which never live in the vault
//! (format in the README).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use kariera_domain::{Personal, Photo, Text, ValueError};
use thiserror::Error;

use crate::value::scalar_text;
use crate::yaml::{self, Entry, Value};

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

fn value<T: FromStr<Err = ValueError>>(entry: &Entry) -> Result<Option<T>, Invalid> {
    if entry.value.is_empty() {
        return Ok(None);
    }
    let problem = |message: String| Invalid {
        line: entry.value.line,
        message: format!("`{}` {message}", entry.key),
    };
    let text = scalar_text(&entry.value).map_err(problem)?;
    text.parse()
        .map(Some)
        .map_err(|error: ValueError| problem(error.to_string()))
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

    use kariera_domain::Personal;

    use super::{PersonalError, PersonalFile};

    const PATH: &str = "/home/robin/.config/kariera/personal.yaml";

    fn parse(text: &str) -> Result<PersonalFile, PersonalError> {
        PersonalFile::parse(text, Path::new(PATH), || Ok(PathBuf::from("/home/robin")))
    }

    #[test]
    fn every_field_is_read() {
        let file = parse(
            "email: robin@example.com\nphone: +81 90 0000 0000\nlocation: Sapporo, Japan\n\
             birth_date: 1990-01-01\nnationality: deutsch\nphoto: photo.jpg\n",
        )
        .unwrap();

        assert_eq!(
            file.personal,
            Personal {
                email: Some("robin@example.com".parse().unwrap()),
                phone: Some("+81 90 0000 0000".parse().unwrap()),
                location: Some("Sapporo, Japan".parse().unwrap()),
                birth_date: Some("1990-01-01".parse().unwrap()),
                nationality: Some("deutsch".parse().unwrap()),
                photo: None,
            }
        );
        assert_eq!(
            file.photo,
            Some(PathBuf::from("/home/robin/.config/kariera/photo.jpg"))
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
