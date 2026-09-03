use chrono::{NaiveDateTime, Offset, TimeZone};
use chrono_tz::Tz;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::Deserialize;
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
    let default_client = Client::builder()
        .user_agent("AstroAgent/1.0")
        .build()
        .map_err(|e| format!("Client Error: {}", e))?;
    get_location_data_with_client(&default_client, city, naive_dt).await
}

pub async fn get_location_data_with_client(
    client: &Client,
    city: &str,
    naive_dt: NaiveDateTime,
) -> Result<(f64, f64, f64), String> {
    let url = reqwest::Url::parse_with_params(
        "https://nominatim.openstreetmap.org/search",
        &[("q", city), ("format", "json"), ("limit", "1")],
    )
    .map_err(|e| format!("Invalid URL construction: {}", e))?;

    let response = client
        .get(url)
        .header("User-Agent", "AstroAgent/1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to reach Nominatim: {}", e))?;

    let results: Vec<NominatimResult> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Nominatim JSON: {}", e))?;

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

    #[test]
    fn test_finder_timezone_resolution() {
        // Test offline timezone lookup via tzf-rs (London: lat ~51.5, lon ~-0.12)
        let tz_name = FINDER.get_tz_name(-0.1276, 51.5074);
        assert_eq!(tz_name, "Europe/London");

        // Tokyo: lat ~35.67, lon ~139.65
        let tz_tokyo = FINDER.get_tz_name(139.6503, 35.6762);
        assert_eq!(tz_tokyo, "Asia/Tokyo");
    }

    #[test]
    fn test_nominatim_url_construction() {
        let city = "New York";
        let url = reqwest::Url::parse_with_params(
            "https://nominatim.openstreetmap.org/search",
            &[("q", city), ("format", "json"), ("limit", "1")],
        )
        .unwrap();
        assert_eq!(
            url.as_str(),
            "https://nominatim.openstreetmap.org/search?q=New+York&format=json&limit=1"
        );
    }
}
