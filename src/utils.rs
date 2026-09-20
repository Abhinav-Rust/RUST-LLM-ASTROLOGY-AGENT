use chrono::{NaiveDate, NaiveTime};

pub fn parse_flexible_date(input: &str) -> Option<NaiveDate> {
    let trimmed = input.trim();
    let formats = ["%d/%m/%Y", "%Y-%m-%d", "%d-%m-%Y", "%m/%d/%Y", "%d.%m.%Y"];

    for fmt in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(trimmed, fmt) {
            return Some(date);
        }
    }
    None
}

pub fn parse_flexible_time(input: &str) -> Option<NaiveTime> {
    let trimmed = input.trim();
    let formats = [
        "%I:%M %p",
        "%I:%M%p",
        "%I:%M:%S %p",
        "%I:%M:%S%p",
        "%H:%M",
        "%H:%M:%S",
    ];

    for fmt in &formats {
        if let Ok(time) = NaiveTime::parse_from_str(trimmed, fmt) {
            return Some(time);
        }
    }
    None
}

pub fn sanitize_filename(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut last_was_underscore = false;

    for c in input.chars() {
        if c.is_alphanumeric() {
            result.push(c);
            last_was_underscore = false;
        } else if !last_was_underscore {
            result.push('_');
            last_was_underscore = true;
        }
    }

    result.trim_matches('_').to_string()
}

pub fn escape_html(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(c),
        }
    }
    result
}

pub fn generate_html_report(name: &str, reading: &str) -> String {
    let safe_name = escape_html(name);
    let safe_reading = escape_html(reading);

    format!(
        "<!DOCTYPE html>\n<html>\n<head>\n\
        <meta charset=\"UTF-8\">\n<title>Vedic Reading - {}</title>\n\
        <style>\n\
        body {{ font-family: system-ui, sans-serif; max-width: 800px; margin: 40px auto; line-height: 1.8; color: #333; padding: 20px; background-color: #fcfcfc; border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.1); }}\n\
        </style>\n</head>\n<body>\n\
        <h1>Vedic Reading for {}</h1>\n\
        <pre style=\"white-space: pre-wrap; font-family: inherit;\">{}</pre>\n\
        </body>\n</html>",
        safe_name, safe_name, safe_reading
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("John Doe"), "John_Doe");
        assert_eq!(sanitize_filename("user@name!"), "user_name");
        assert_eq!(sanitize_filename("123 test"), "123_test");
        assert_eq!(sanitize_filename(""), "");
        assert_eq!(sanitize_filename("!@#$%^&*()"), "");
        assert_eq!(sanitize_filename("  spaces  "), "spaces");
        assert_eq!(sanitize_filename("Alice & Bob-Smith"), "Alice_Bob_Smith");
        assert_eq!(sanitize_filename("John__Doe"), "John_Doe");
        assert_eq!(sanitize_filename("---test---"), "test");
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(
            escape_html("Hello <World> & 'Friends' \"123\""),
            "Hello &lt;World&gt; &amp; &#39;Friends&#39; &quot;123&quot;"
        );
        assert_eq!(escape_html("Plain Text"), "Plain Text");
        assert_eq!(escape_html(""), "");
    }

    #[test]
    fn test_generate_html_report() {
        let html = generate_html_report("Jane & John <Doe>", "Line 1\nLine 2 & <More>");
        assert!(html.contains("<title>Vedic Reading - Jane &amp; John &lt;Doe&gt;</title>"));
        assert!(html.contains("<h1>Vedic Reading for Jane &amp; John &lt;Doe&gt;</h1>"));
        assert!(html.contains("Line 1\nLine 2 &amp; &lt;More&gt;"));
    }

    #[test]
    fn test_parse_flexible_date() {
        assert_eq!(
            parse_flexible_date("15/08/1990"),
            NaiveDate::from_ymd_opt(1990, 8, 15)
        );
        assert_eq!(
            parse_flexible_date("1990-08-15"),
            NaiveDate::from_ymd_opt(1990, 8, 15)
        );
        assert_eq!(
            parse_flexible_date("15-08-1990"),
            NaiveDate::from_ymd_opt(1990, 8, 15)
        );
        assert_eq!(
            parse_flexible_date("08/15/1990"),
            NaiveDate::from_ymd_opt(1990, 8, 15)
        );
        assert_eq!(
            parse_flexible_date("15.08.1990"),
            NaiveDate::from_ymd_opt(1990, 8, 15)
        );
        assert_eq!(parse_flexible_date("invalid date"), None);
    }

    #[test]
    fn test_parse_flexible_time() {
        assert_eq!(
            parse_flexible_time("10:45 AM"),
            NaiveTime::from_hms_opt(10, 45, 0)
        );
        assert_eq!(
            parse_flexible_time("10:45AM"),
            NaiveTime::from_hms_opt(10, 45, 0)
        );
        assert_eq!(
            parse_flexible_time("14:30"),
            NaiveTime::from_hms_opt(14, 30, 0)
        );
        assert_eq!(
            parse_flexible_time("14:30:15"),
            NaiveTime::from_hms_opt(14, 30, 15)
        );
        assert_eq!(parse_flexible_time("invalid time"), None);
    }
}
