use chrono::{NaiveDateTime, Offset, TimeZone};
use chrono_tz::Tz;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;
use tzf_rs::DefaultFinder;

static FINDER: Lazy<DefaultFinder> = Lazy::new(DefaultFinder::new);

#[derive(Deserialize)]
struct NominatimResult {
    lat: String,
    lon: String,
}

pub async fn get_location_data(
    city: &str,
    naive_dt: NaiveDateTime,
) -> Result<(f64, f64, f64), String> {
    let client = Client::builder()
        .user_agent("AstroAgent/1.0")
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Client Error: {}", e))?;

    let url = format!(
        "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=1",
        city
    );

    let mut attempts = 0;
    let max_attempts = 3;
    let mut last_err = format!("Failed to reach Nominatim for {}", city);

    let results: Vec<NominatimResult> = loop {
        attempts += 1;
        match client.get(&url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    last_err = format!("Nominatim returned status {}", response.status());
                } else {
                    match response.json::<Vec<NominatimResult>>().await {
                        Ok(parsed) => break parsed,
                        Err(e) => {
                            last_err = format!("Failed to parse Nominatim response: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                last_err = format!("Failed to reach Nominatim: {}", e);
            }
        }

        if attempts >= max_attempts {
            return Err(last_err);
        }
        sleep(Duration::from_millis(500 * attempts as u64)).await;
    };

    let result = results
        .first()
        .ok_or_else(|| format!("City not found: {}", city))?;

    let lat: f64 = result
        .lat
        .parse()
        .map_err(|_| "Invalid latitude from API")?;
    let lon: f64 = result
        .lon
        .parse()
        .map_err(|_| "Invalid longitude from API")?;

    // tzf-rs takes (longitude, latitude)
    let tz_name = FINDER.get_tz_name(lon, lat);

    println!("Resolved Location: {} -> ({}, {})", city, lat, lon);
    println!("Resolved Timezone: {}", tz_name);

    let tz: Tz = tz_name
        .parse()
        .map_err(|_| format!("Unsupported timezone: {}", tz_name))?;

    // Get historical UTC offset
    let dt = tz
        .from_local_datetime(&naive_dt)
        .earliest()
        .ok_or_else(|| {
            "The provided local time is non-existent due to a DST transition.".to_string()
        })?;

    let offset_seconds = dt.offset().fix().local_minus_utc();
    let offset_hours = offset_seconds as f64 / 3600.0;

    println!("Historical UTC Offset: {:.2} hours", offset_hours);

    Ok((lat, lon, offset_hours))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_tz_finder_resolution() {
        // London coordinates roughly (51.5074, -0.1278) -> lon=-0.1278, lat=51.5074
        let tz_name = FINDER.get_tz_name(-0.1278, 51.5074);
        assert!(!tz_name.is_empty());
        let parsed: Result<Tz, _> = tz_name.parse();
        assert!(parsed.is_ok());

        let naive_dt = NaiveDate::from_ymd_opt(2023, 6, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let tz = parsed.unwrap();
        let dt = tz.from_local_datetime(&naive_dt).earliest().unwrap();
        let offset_hours = dt.offset().fix().local_minus_utc() as f64 / 3600.0;
        assert_eq!(offset_hours, 1.0); // BST is UTC+1
    }
}
