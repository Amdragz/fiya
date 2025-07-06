use std::{env, io::Cursor};

use axum_extra::headers::UserAgent;
use chrono::{DateTime, Utc};
use dotenvy::dotenv;
use genpdf::{
    elements::{self, Paragraph, StyledElement, TableLayout},
    fonts::from_files,
    style::Style,
    Document,
};
use hmac::{Hmac, Mac};
use project_root::get_project_root;
use rand::{distr::Alphanumeric, Rng};
use sha2::Sha256;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::spm::Cage;

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
    common_browsers
        .iter()
        .any(|&browser| user_agent.as_str().contains(browser))
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

pub fn generate_secure_device_token() -> (String, String) {
    dotenv().ok();
    let spm_secret = env::var("SPM_SECRET").expect("SPM_SECRET must be set");

    let uuid = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let salt: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let temp_token = format!("{}:{}:{}", uuid, now, salt);

    let mut mac1 = Hmac::<Sha256>::new_from_slice(spm_secret.as_bytes())
        .expect("hmac can only accept secret of a perticular length");
    mac1.update(temp_token.as_bytes());
    let result1 = mac1.finalize().into_bytes();
    let device_token = hex::encode(result1);

    let mut mac2 = Hmac::<Sha256>::new_from_slice(spm_secret.as_bytes())
        .expect("hmac can only accept secret of a perticular length");
    mac2.update(device_token.as_bytes());
    let result2 = mac2.finalize().into_bytes();
    let hashed_token = hex::encode(result2);

    (device_token, hashed_token)
}

pub fn generate_pdf_for_cage_data(cages: Vec<Cage>) -> Result<Vec<u8>, genpdf::error::Error> {
    let root_path = get_project_root().expect("Failed to get project root");
    let font_path = root_path.join("fonts");

    let font_family =
        from_files(font_path, "LiberationSans", None).expect("Failed to load font family");

    let mut doc = Document::new(font_family);
    doc.set_title("Smart poultry monitor cage data");

    let column_widths = vec![2, 3, 4, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 6, 6, 6];

    let mut table = TableLayout::new(column_widths);
    table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));

    let header_style = Style::new().bold();
    table
        .row()
        .element(StyledElement::new(Paragraph::new("ID"), header_style))
        .element(Paragraph::new("Cage ID"))
        .element(Paragraph::new("Assigned Monitor").aligned(genpdf::Alignment::Center))
        .element(Paragraph::new("Livestock No").aligned(genpdf::Alignment::Center))
        .element(Paragraph::new("Temperature").aligned(genpdf::Alignment::Center))
        .element(Paragraph::new("Humidity").aligned(genpdf::Alignment::Center))
        .element(Paragraph::new("Pressure"))
        .element(Paragraph::new("Ammonia"))
        .element(Paragraph::new("CO2"))
        .element(Paragraph::new("Coccidiosis").aligned(genpdf::Alignment::Center))
        .element(Paragraph::new("Newcastle").aligned(genpdf::Alignment::Center))
        .element(Paragraph::new("Salmonella").aligned(genpdf::Alignment::Center))
        .element(Paragraph::new("Healthy"))
        .element(Paragraph::new("Timestamp").aligned(genpdf::Alignment::Center))
        .element(Paragraph::new("Created At").aligned(genpdf::Alignment::Center))
        .element(Paragraph::new("Updated At").aligned(genpdf::Alignment::Center))
        .push()?;

    cages.iter().try_for_each(|cage| {
        table
            .row()
            .element(Paragraph::new(cage.id.to_string()))
            .element(Paragraph::new(&cage.cage_id))
            .element(Paragraph::new(&cage.assigned_monitor))
            .element(Paragraph::new(cage.livestock_no.to_string()))
            .element(Paragraph::new(cage.temperature.to_string()))
            .element(Paragraph::new(cage.humidity.to_string()))
            .element(Paragraph::new(cage.pressure.to_string()))
            .element(Paragraph::new(cage.ammonia.to_string()))
            .element(Paragraph::new(cage.co2.to_string()))
            .element(Paragraph::new(
                cage.object_recognition.coccidiosis.to_string(),
            ))
            .element(Paragraph::new(
                cage.object_recognition.newcastle.to_string(),
            ))
            .element(Paragraph::new(
                cage.object_recognition.salmonella.to_string(),
            ))
            .element(Paragraph::new(cage.object_recognition.healthy.to_string()))
            .element(Paragraph::new(cage.timestamp.to_rfc2822()))
            .element(Paragraph::new(cage.created_at.to_rfc2822()))
            .element(Paragraph::new(cage.updated_at.to_rfc2822()))
            .push()
    })?;

    doc.push(table);
    let mut buffer = Cursor::new(Vec::new());
    doc.render(&mut buffer)?;
    Ok(buffer.into_inner())
}
