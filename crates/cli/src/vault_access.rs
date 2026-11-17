use std::path::PathBuf;

use kariera_vault::{Config, Vault};

/// The root of the vault the config names. The error is what to tell the user.
pub(crate) fn vault_root() -> Result<PathBuf, String> {
    let config_path = Config::default_path().map_err(|error| error.to_string())?;
    let config = Config::load(&config_path).map_err(|error| error.to_string())?;
    Ok(config.vault().to_owned())
}

/// The vault the config names, loaded. The error is what to tell the user.
pub(crate) fn open_vault() -> Result<Vault, String> {
    Vault::load(&vault_root()?).map_err(|error| error.to_string())
}
