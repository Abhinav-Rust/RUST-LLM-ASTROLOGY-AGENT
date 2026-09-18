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

#[derive(Debug, Clone, PartialEq)]
pub struct LocationData {
    pub latitude: f64,
    pub longitude: f64,
    pub utc_offset_hours: f64,
}

pub async fn get_location_data(
    client: &Client,
    city: &str,
    naive_dt: NaiveDateTime,
) -> Result<LocationData, String> {
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

    Ok(LocationData {
        latitude: lat,
        longitude: lon,
        utc_offset_hours: offset_hours,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_location_data_struct() {
        let loc = LocationData {
            latitude: 51.5074,
            longitude: -0.1278,
            utc_offset_hours: 1.0,
        };
        assert_eq!(loc.latitude, 51.5074);
        assert_eq!(loc.longitude, -0.1278);
        assert_eq!(loc.utc_offset_hours, 1.0);
    }

    #[test]
    fn test_timezone_finder_lookups() {
        // London
        assert_eq!(FINDER.get_tz_name(-0.1278, 51.5074), "Europe/London");
        // Tokyo
        assert_eq!(FINDER.get_tz_name(139.6917, 35.6895), "Asia/Tokyo");
        // New York
        assert_eq!(FINDER.get_tz_name(-74.0060, 40.7128), "America/New_York");
    }

    #[test]
    fn test_historical_timezone_offset_calculation() {
        let tz: Tz = "Europe/London".parse().unwrap();
        let winter_dt = NaiveDate::from_ymd_opt(2025, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let summer_dt = NaiveDate::from_ymd_opt(2025, 7, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();

        let winter_offset = tz
            .from_local_datetime(&winter_dt)
            .earliest()
            .unwrap()
            .offset()
            .fix()
            .local_minus_utc();
        let summer_offset = tz
            .from_local_datetime(&summer_dt)
            .earliest()
            .unwrap()
            .offset()
            .fix()
            .local_minus_utc();

        assert_eq!(winter_offset, 0); // London Winter UTC+0
        assert_eq!(summer_offset, 3600); // London Summer BST UTC+1
    }
}
