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
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn test_timezone_resolution() {
        // Test coordinates for London (51.5074, -0.1278)
        let tz_name = FINDER.get_tz_name(-0.1278, 51.5074);
        assert_eq!(tz_name, "Europe/London");

        let tz: Tz = tz_name.parse().unwrap();
        let naive_dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        );
        let dt = tz.from_local_datetime(&naive_dt).earliest().unwrap();
        let offset_hours = dt.offset().fix().local_minus_utc() as f64 / 3600.0;
        assert_eq!(offset_hours, 0.0); // GMT in winter
    }
}
