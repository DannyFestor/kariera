use std::time::{SystemTime, UNIX_EPOCH};

use kariera_domain::Month;

/// The month it is now, by the system clock. The error is what to tell the user.
pub(crate) fn current_month() -> Result<Month, String> {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("the system clock is set before 1970: {error}"))?;
    Month::of_unix_seconds(since_epoch.as_secs())
        .ok_or_else(|| "the system clock is set beyond the year 65535".to_owned())
}
