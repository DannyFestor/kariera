use kariera_vault::{Config, Vault};

/// The vault the config names, loaded. The error is what to tell the user.
pub(crate) fn open_vault() -> Result<Vault, String> {
    let config_path = Config::default_path().map_err(|error| error.to_string())?;
    let config = Config::load(&config_path).map_err(|error| error.to_string())?;
    Vault::load(config.vault()).map_err(|error| error.to_string())
}
