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
    client: &Client,
    city: &str,
    naive_dt: NaiveDateTime,
) -> Result<(f64, f64, f64), String> {
    let response = client
        .get("https://nominatim.openstreetmap.org/search")
        .header("User-Agent", "AstroAgent/1.0")
        .query(&[("q", city), ("format", "json"), ("limit", "1")])
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
    fn test_nominatim_result_deserialization() {
        let json_data = r#"[
            {
                "lat": "51.5073219",
                "lon": "-0.1276474"
            }
        ]"#;

        let results: Result<Vec<NominatimResult>, _> = serde_json::from_str(json_data);
        assert!(results.is_ok());
        let items = results.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].lat, "51.5073219");
        assert_eq!(items[0].lon, "-0.1276474");

        let lat: f64 = items[0].lat.parse().unwrap();
        let lon: f64 = items[0].lon.parse().unwrap();
        assert!((lat - 51.5073219).abs() < 1e-6);
        assert!((lon - (-0.1276474)).abs() < 1e-6);
    }

    #[test]
    fn test_nominatim_result_empty_or_invalid() {
        let empty_json = "[]";
        let results: Vec<NominatimResult> = serde_json::from_str(empty_json).unwrap();
        assert!(results.is_empty());

        let invalid_json = r#"[{"lat": "not_a_number", "lon": "0.0"}]"#;
        let results: Vec<NominatimResult> = serde_json::from_str(invalid_json).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].lat.parse::<f64>().is_err());
    }
}
