use chrono::{NaiveDate, NaiveTime};

pub fn parse_flexible_date(input: &str) -> Option<NaiveDate> {
    let trimmed = input.trim();
    let formats = ["%d/%m/%Y", "%Y-%m-%d", "%d-%m-%Y", "%m/%d/%Y", "%d.%m.%Y"];
    for fmt in &formats {
        if let Ok(dt) = NaiveDate::parse_from_str(trimmed, fmt) {
            return Some(dt);
        }
    }
    None
}

pub fn parse_flexible_time(input: &str) -> Option<NaiveTime> {
    let trimmed = input.trim();
    let formats = [
        "%I:%M %p", "%I:%M%p", "%H:%M", "%H:%M:%S", "%l:%M %p", "%l:%M%p",
    ];
    for fmt in &formats {
        if let Ok(t) = NaiveTime::parse_from_str(trimmed, fmt) {
            return Some(t);
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

pub fn format_reading_html(reading: &str) -> String {
    let mut html_blocks = Vec::new();
    let blocks = reading.split("\n\n");

    for block in blocks {
        let trimmed = block.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "---" || trimmed == "***" {
            html_blocks.push("<hr>".to_string());
        } else if let Some(content) = trimmed.strip_prefix("### ") {
            html_blocks.push(format!("<h3>{}</h3>", escape_html(content.trim())));
        } else if let Some(content) = trimmed.strip_prefix("## ") {
            html_blocks.push(format!("<h2>{}</h2>", escape_html(content.trim())));
        } else if let Some(content) = trimmed.strip_prefix("# ") {
            html_blocks.push(format!("<h1>{}</h1>", escape_html(content.trim())));
        } else {
            let lines: Vec<String> = trimmed
                .lines()
                .map(|line| escape_html(line.trim_end()))
                .collect();
            let paragraph_body = lines.join("<br>\n");
            html_blocks.push(format!("<p>{}</p>", paragraph_body));
        }
    }

    html_blocks.join("\n")
}

pub fn generate_html_report(name: &str, reading: &str) -> String {
    let safe_name = escape_html(name);
    let formatted_reading = format_reading_html(reading);
    let generated_at = chrono::Local::now()
        .format("%d %B %Y, %H:%M:%S %Z")
        .to_string();

    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n\
        <meta charset=\"UTF-8\">\n\
        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n\
        <title>Vedic Astrological Reading — {}</title>\n\
        <style>\n\
        :root {{ --primary: #1e293b; --accent: #6366f1; --bg: #f8fafc; --card-bg: #ffffff; --text: #334155; --border: #e2e8f0; }}\n\
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; max-width: 860px; margin: 40px auto; line-height: 1.8; color: var(--text); padding: 24px; background-color: var(--bg); }}\n\
        .header {{ background: linear-gradient(135deg, #1e1b4b 0%, #312e81 100%); color: #ffffff; padding: 32px; border-radius: 12px 12px 0 0; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1); }}\n\
        .header h1 {{ margin: 0 0 8px 0; font-size: 28px; font-weight: 700; letter-spacing: -0.02em; color: #ffffff; }}\n\
        .header .meta {{ font-size: 14px; color: #c7d2fe; opacity: 0.9; }}\n\
        .content {{ background: var(--card-bg); padding: 36px; border-radius: 0 0 12px 12px; border: 1px solid var(--border); border-top: none; box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.05); }}\n\
        .content h1 {{ color: #1e1b4b; font-size: 24px; margin-top: 24px; margin-bottom: 12px; border-bottom: 2px solid var(--border); padding-bottom: 6px; }}\n\
        .content h2 {{ color: #312e81; font-size: 20px; margin-top: 20px; margin-bottom: 10px; }}\n\
        .content h3 {{ color: var(--accent); font-size: 16px; margin-top: 16px; margin-bottom: 8px; }}\n\
        .content p {{ margin: 0 0 16px 0; color: var(--text); font-size: 16px; }}\n\
        .content hr {{ border: none; border-top: 1px solid var(--border); margin: 24px 0; }}\n\
        .footer {{ text-align: center; margin-top: 24px; font-size: 13px; color: #94a3b8; }}\n\
        @media (max-width: 640px) {{ body {{ padding: 12px; margin: 10px auto; }} .header, .content {{ padding: 20px; }} }}\n\
        </style>\n</head>\n<body>\n\
        <div class=\"header\">\n\
        <h1>Vedic Astrological Analysis</h1>\n\
        <div class=\"meta\">Querent: <strong>{}</strong> | Generated: {}</div>\n\
        </div>\n\
        <div class=\"content\">\n\
        {}\n\
        </div>\n\
        <div class=\"footer\">Generated by Rust LLM Astrology Engine • Confidential</div>\n\
        </body>\n</html>",
        safe_name, safe_name, generated_at, formatted_reading
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
    fn test_parse_flexible_date() {
        assert_eq!(
            parse_flexible_date("15/08/1990"),
            Some(NaiveDate::from_ymd_opt(1990, 8, 15).unwrap())
        );
        assert_eq!(
            parse_flexible_date("1990-08-15"),
            Some(NaiveDate::from_ymd_opt(1990, 8, 15).unwrap())
        );
        assert_eq!(
            parse_flexible_date("15-08-1990"),
            Some(NaiveDate::from_ymd_opt(1990, 8, 15).unwrap())
        );
        assert_eq!(
            parse_flexible_date("08/15/1990"),
            Some(NaiveDate::from_ymd_opt(1990, 8, 15).unwrap())
        );
        assert_eq!(
            parse_flexible_date("15.08.1990"),
            Some(NaiveDate::from_ymd_opt(1990, 8, 15).unwrap())
        );
        assert_eq!(parse_flexible_date("invalid-date"), None);
    }

    #[test]
    fn test_parse_flexible_time() {
        assert_eq!(
            parse_flexible_time("10:45 AM"),
            Some(NaiveTime::from_hms_opt(10, 45, 0).unwrap())
        );
        assert_eq!(
            parse_flexible_time("10:45AM"),
            Some(NaiveTime::from_hms_opt(10, 45, 0).unwrap())
        );
        assert_eq!(
            parse_flexible_time("14:30"),
            Some(NaiveTime::from_hms_opt(14, 30, 0).unwrap())
        );
        assert_eq!(
            parse_flexible_time("14:30:15"),
            Some(NaiveTime::from_hms_opt(14, 30, 15).unwrap())
        );
        assert_eq!(parse_flexible_time("invalid-time"), None);
    }

    #[test]
    fn test_format_reading_html() {
        let input = "# Heading 1\n\n## Heading 2\n\n### Heading 3\n\n---\n\nParagraph text with <tags> & 'quotes'.\nLine 2 of paragraph.";
        let formatted = format_reading_html(input);

        assert!(formatted.contains("<h1>Heading 1</h1>"));
        assert!(formatted.contains("<h2>Heading 2</h2>"));
        assert!(formatted.contains("<h3>Heading 3</h3>"));
        assert!(formatted.contains("<hr>"));
        assert!(formatted.contains("<p>Paragraph text with &lt;tags&gt; &amp; &#39;quotes&#39;.<br>\nLine 2 of paragraph.</p>"));
    }

    #[test]
    fn test_generate_html_report() {
        let html =
            generate_html_report("Jane & John <Doe>", "# Reading\n\nLine 1\nLine 2 & <More>");
        assert!(
            html.contains(
                "<title>Vedic Astrological Reading — Jane &amp; John &lt;Doe&gt;</title>"
            )
        );
        assert!(html.contains("Querent: <strong>Jane &amp; John &lt;Doe&gt;</strong>"));
        assert!(html.contains("<h1>Reading</h1>"));
        assert!(html.contains("<p>Line 1<br>\nLine 2 &amp; &lt;More&gt;</p>"));
    }
}
