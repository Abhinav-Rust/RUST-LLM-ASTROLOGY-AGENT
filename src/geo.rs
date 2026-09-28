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

    println!("Resolved Location: {} -> ({}, {})", city, lat, lon);

    let (tz_name, offset_hours) = resolve_timezone_and_offset(lat, lon, naive_dt)?;

    println!("Resolved Timezone: {}", tz_name);
    println!("Historical UTC Offset: {:.2} hours", offset_hours);

    Ok(LocationData {
        latitude: lat,
        longitude: lon,
        utc_offset_hours: offset_hours,
    })
}

pub fn resolve_timezone_and_offset(
    lat: f64,
    lon: f64,
    naive_dt: NaiveDateTime,
) -> Result<(String, f64), String> {
    // tzf-rs takes (longitude, latitude)
    let tz_name = FINDER.get_tz_name(lon, lat);

    let tz: Tz = tz_name
        .parse()
        .map_err(|_| format!("Unsupported timezone: {}", tz_name))?;

    let dt = tz
        .from_local_datetime(&naive_dt)
        .earliest()
        .ok_or_else(|| {
            "The provided local time is non-existent due to a DST transition.".to_string()
        })?;

    let offset_seconds = dt.offset().fix().local_minus_utc();
    let offset_hours = offset_seconds as f64 / 3600.0;

    Ok((tz_name.to_string(), offset_hours))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_timezone_and_offset() {
        let dt = NaiveDateTime::parse_from_str("2025-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        // London in summer (BST = UTC+1)
        let (tz_london, offset_london) = resolve_timezone_and_offset(51.5074, -0.1278, dt).unwrap();
        assert_eq!(tz_london, "Europe/London");
        assert_eq!(offset_london, 1.0);

        // Tokyo (JST = UTC+9)
        let (tz_tokyo, offset_tokyo) = resolve_timezone_and_offset(35.6762, 139.6503, dt).unwrap();
        assert_eq!(tz_tokyo, "Asia/Tokyo");
        assert_eq!(offset_tokyo, 9.0);

        // New York in winter (EST = UTC-5)
        let dt_winter =
            NaiveDateTime::parse_from_str("2025-01-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let (tz_ny, offset_ny) = resolve_timezone_and_offset(40.7128, -74.0060, dt_winter).unwrap();
        assert_eq!(tz_ny, "America/New_York");
        assert_eq!(offset_ny, -5.0);

        // Sydney in winter (AEST = UTC+10)
        let (tz_sydney, offset_sydney) =
            resolve_timezone_and_offset(-33.8688, 151.2093, dt).unwrap();
        assert_eq!(tz_sydney, "Australia/Sydney");
        assert_eq!(offset_sydney, 10.0);
    }

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
}
