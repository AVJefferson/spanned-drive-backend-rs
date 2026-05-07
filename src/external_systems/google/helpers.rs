/// Encodes an email address for use in an appData filename.
///
/// These substitutions MUST stay byte-for-byte compatible with previously
/// uploaded files, otherwise existing entries become unreadable.
pub(super) fn encode_email_for_filename(email: &str) -> String {
    email.replace("@", "__at__").replace(".", "__dot__")
}

/// Encodes a user-supplied drive/folder name for use in an appData filename.
///
/// These substitutions MUST stay byte-for-byte compatible with previously
/// uploaded files, otherwise existing entries become unreadable.
pub(super) fn encode_drive_name(name: &str) -> String {
    name.replace("@", "__at__")
        .replace(".", "__dot__")
        .replace("/", "__slash__")
        .replace("\\", "__backslash__")
        .replace(" ", "__space__")
}

pub(super) fn secondary_drive_filename(
    primary_email_encoded: &str,
    drive_provider: &str,
    secondary_email_encoded: &str,
) -> String {
    format!(
        "sdrive---secondary-drive---{}---{}---{}.json",
        primary_email_encoded, drive_provider, secondary_email_encoded
    )
}

pub(super) fn logical_folder_filename(
    primary_email_encoded: &str,
    folder_name_encoded: &str,
) -> String {
    format!(
        "sdrive---logical-folder---{}---{}.json",
        primary_email_encoded, folder_name_encoded
    )
}

/// Returns a hex-encoded nanosecond timestamp. Used as a unique suffix for
/// multipart upload boundaries; NOT a standards-compliant UUID v4.
pub(super) fn timestamp_hex() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", t)
}
