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
    use chrono::NaiveDate;

    #[test]
    fn test_nominatim_result_deserialization() {
        let json_data = r#"[{"lat": "51.5074", "lon": "-0.1278"}]"#;
        let results: Vec<NominatimResult> = serde_json::from_str(json_data).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].lat, "51.5074");
        assert_eq!(results[0].lon, "-0.1278");
    }

    #[test]
    fn test_timezone_finder_lookups() {
        // London: 51.5074 N, 0.1278 W -> lon=-0.1278, lat=51.5074
        let london_tz = FINDER.get_tz_name(-0.1278, 51.5074);
        assert_eq!(london_tz, "Europe/London");

        // Tokyo: 35.6762 N, 139.6503 E -> lon=139.6503, lat=35.6762
        let tokyo_tz = FINDER.get_tz_name(139.6503, 35.6762);
        assert_eq!(tokyo_tz, "Asia/Tokyo");

        // New York: 40.7128 N, 74.0060 W -> lon=-74.0060, lat=40.7128
        let ny_tz = FINDER.get_tz_name(-74.0060, 40.7128);
        assert_eq!(ny_tz, "America/New_York");
    }

    #[test]
    fn test_historical_offset_calculation() {
        let london_tz_name = FINDER.get_tz_name(-0.1278, 51.5074);
        let tz: Tz = london_tz_name.parse().unwrap();

        // Winter time (GMT, offset 0.0)
        let winter_dt = NaiveDate::from_ymd_opt(2023, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let dt_winter = tz.from_local_datetime(&winter_dt).earliest().unwrap();
        let offset_winter_hours = dt_winter.offset().fix().local_minus_utc() as f64 / 3600.0;
        assert_eq!(offset_winter_hours, 0.0);

        // Summer time (BST, offset +1.0)
        let summer_dt = NaiveDate::from_ymd_opt(2023, 7, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let dt_summer = tz.from_local_datetime(&summer_dt).earliest().unwrap();
        let offset_summer_hours = dt_summer.offset().fix().local_minus_utc() as f64 / 3600.0;
        assert_eq!(offset_summer_hours, 1.0);
    }
}
