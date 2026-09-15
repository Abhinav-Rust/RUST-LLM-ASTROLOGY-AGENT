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
}
