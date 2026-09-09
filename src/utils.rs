pub fn sanitize_filename(input: &str) -> String {
    let replaced: String = input
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();

    let mut result = String::new();
    let mut prev_is_underscore = false;

    for c in replaced.chars() {
        if c == '_' {
            if !prev_is_underscore {
                result.push(c);
                prev_is_underscore = true;
            }
        } else {
            result.push(c);
            prev_is_underscore = false;
        }
    }

    result.trim_matches('_').to_string()
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
    }
}
