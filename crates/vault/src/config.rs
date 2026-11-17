use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

/// kariera's settings, from `~/.config/kariera/config.toml` (format in the README).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    vault: PathBuf,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("no home directory is known, so there is no config path")]
    NoHome,
    #[error("cannot read {}: {source}", path.display())]
    Unreadable { path: PathBuf, source: io::Error },
    #[error("{} is not a valid kariera config: {source}", path.display())]
    Invalid {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("{}: `vault` must be an absolute path or start with ~/, not {value:?}", path.display())]
    RelativeVault { path: PathBuf, value: String },
}

/// The file as written. Unknown keys are refused, so a misspelt setting is not silently ignored.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    vault: String,
}

impl Config {
    pub fn default_path() -> Result<PathBuf, ConfigError> {
        home().map(|home| home.join(".config/kariera/config.toml"))
    }

    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let text = fs::read_to_string(path).map_err(|source| ConfigError::Unreadable {
            path: path.to_owned(),
            source,
        })?;
        Self::parse(&text, path, home)
    }

    /// The vault root: the folder that holds `Profile.md` and the note folders.
    #[must_use]
    pub fn vault(&self) -> &Path {
        &self.vault
    }

    /// `home` is asked only when `vault` starts with `~/`.
    fn parse(
        text: &str,
        path: &Path,
        home: impl FnOnce() -> Result<PathBuf, ConfigError>,
    ) -> Result<Self, ConfigError> {
        let file: ConfigFile = toml::from_str(text).map_err(|source| ConfigError::Invalid {
            path: path.to_owned(),
            source,
        })?;
        let vault = match file.vault.strip_prefix("~/") {
            Some(in_home) => home()?.join(in_home),
            None => PathBuf::from(&file.vault),
        };
        if vault.is_absolute() {
            Ok(Self { vault })
        } else {
            Err(ConfigError::RelativeVault {
                path: path.to_owned(),
                value: file.vault,
            })
        }
    }
}

fn home() -> Result<PathBuf, ConfigError> {
    std::env::home_dir().ok_or(ConfigError::NoHome)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{Config, ConfigError};

    const CONFIG: &str = "/home/robin/.config/kariera/config.toml";

    fn parse(text: &str) -> Result<Config, ConfigError> {
        Config::parse(text, Path::new(CONFIG), || Ok(PathBuf::from("/home/robin")))
    }

    #[test]
    fn an_absolute_vault_path_is_kept() {
        let config = parse("vault = \"/srv/notes/Career\"\n").unwrap();

        assert_eq!(config.vault(), Path::new("/srv/notes/Career"));
    }

    #[test]
    fn a_vault_path_in_the_home_directory_is_expanded() {
        let config = parse("vault = \"~/Notes/Career\"\n").unwrap();

        assert_eq!(config.vault(), Path::new("/home/robin/Notes/Career"));
    }

    #[test]
    fn a_relative_vault_path_is_refused() {
        let error = parse("vault = \"Notes/Career\"\n").unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "{CONFIG}: `vault` must be an absolute path or start with ~/, not \"Notes/Career\""
            )
        );
    }

    #[test]
    fn a_missing_or_unknown_key_is_refused() {
        for text in ["", "vault = \"/srv\"\nvalut = \"/srv\"\n"] {
            let error = parse(text).unwrap_err();

            assert!(matches!(error, ConfigError::Invalid { .. }), "{text:?}");
            assert!(
                error
                    .to_string()
                    .starts_with(&format!("{CONFIG} is not a valid kariera config: ")),
                "{error}"
            );
        }
    }

    #[test]
    fn without_a_home_a_home_path_cannot_be_expanded() {
        let error = Config::parse("vault = \"~/Notes\"\n", Path::new(CONFIG), || {
            Err(ConfigError::NoHome)
        });

        assert!(matches!(error, Err(ConfigError::NoHome)));
    }

    #[test]
    fn a_missing_config_file_cannot_be_read() {
        let error = Config::load(Path::new("/nonexistent/kariera/config.toml")).unwrap_err();

        assert!(
            error
                .to_string()
                .starts_with("cannot read /nonexistent/kariera/config.toml: "),
            "{error}"
        );
    }

    #[test]
    fn the_default_path_is_in_the_home_config_directory() {
        let path = Config::default_path().unwrap();

        assert!(
            path.ends_with(".config/kariera/config.toml"),
            "{}",
            path.display()
        );
        assert!(path.is_absolute(), "{}", path.display());
    }
}
