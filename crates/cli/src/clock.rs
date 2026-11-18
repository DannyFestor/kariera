use kariera_domain::{Date, Month};
use time::OffsetDateTime;

/// The month it is now in the machine's time zone. The error is what to tell the user.
pub(crate) fn current_month() -> Result<Month, String> {
    today().map(Date::month)
}

/// The date it is now in the machine's time zone: the owner reads a document's date (作成日) as a local date.
/// The error is what to tell the user.
pub(crate) fn today() -> Result<Date, String> {
    let now = OffsetDateTime::now_local()
        .map_err(|error| format!("cannot tell the local time zone: {error}"))?;
    u16::try_from(now.year())
        .ok()
        .and_then(|year| Date::from_parts(year, u8::from(now.month()), now.day()))
        .ok_or_else(|| {
            format!(
                "the system clock is set to an unusable year: {}",
                now.year()
            )
        })
}
