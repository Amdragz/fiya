use std::env;

use axum_extra::headers::UserAgent;
use chrono::{DateTime, Utc};
use dotenvy::dotenv;
use hmac::{Hmac, Mac};
use rand::{distr::Alphanumeric, Rng};
use sha2::Sha256;
use time::OffsetDateTime;

pub fn generate_password(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn datetime_to_offset_datetime(datetime: DateTime<Utc>) -> Option<OffsetDateTime> {
    // Convert the `DateTime<Utc>` to a timestamp (seconds since epoch)
    let timestamp = datetime.timestamp();

    // Create an `OffsetDateTime` from the timestamp
    OffsetDateTime::from_unix_timestamp(timestamp).ok()
}

pub fn is_browser(user_agent: UserAgent) -> bool {
    // Define common browser identifiers
    let common_browsers = ["Chrome", "Firefox", "Safari", "Edge", "Opera"];

    // Check if the user agent contains any common browser identifiers
    let is_browser = common_browsers
        .iter()
        .any(|&browser| user_agent.as_str().contains(browser));

    is_browser
}

pub fn hash_id_with_secret(id: &str) -> String {
    dotenv().ok();
    let spm_secret = env::var("SPM_SECRET").expect("SPM_SECRET must be set");
    let mut mac = Hmac::<Sha256>::new_from_slice(spm_secret.as_bytes())
        .expect("Hmac can only accept secrets of a particular length");

    mac.update(id.as_bytes());
    let result = mac.finalize();

    let result_bytes = result.into_bytes();
    hex::encode(result_bytes)
}
