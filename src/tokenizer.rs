pub fn tokenize(input: &str) -> anyhow::Result<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut has_token = false;
    let mut chars = input.trim().chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // --- Single-quoted string: everything is literal until closing ' ---
            '\'' => {
                has_token = true;
                loop {
                    match chars.next() {
                        Some('\'') => break,
                        Some(ch) => current_token.push(ch),
                        None => anyhow::bail!("Unterminated single quote"),
                    }
                }
            }
            // --- Double-quoted string: literal except \\ \" \$ \` \newline ---
            '"' => {
                has_token = true;
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') => {
                            match chars.peek() {
                                Some('"') | Some('\\') | Some('$') | Some('`') | Some('\n') => {
                                    current_token.push(chars.next().unwrap());
                                }
                                _ => {
                                    // Backslash is literal when not followed by a special char
                                    current_token.push('\\');
                                }
                            }
                        }
                        Some(ch) => current_token.push(ch),
                        None => anyhow::bail!("Unterminated double quote"),
                    }
                }
            }
            // --- Unquoted backslash: next char is literal ---
            '\\' => {
                has_token = true;
                match chars.next() {
                    Some(ch) => current_token.push(ch),
                    None => anyhow::bail!("Trailing backslash"),
                }
            }
            // --- Unquoted whitespace: finalize token ---
            ' ' | '\t' => {
                if has_token {
                    tokens.push(current_token);
                    current_token = String::new();
                    has_token = false;
                }
            }
            // --- Normal character ---
            _ => {
                has_token = true;
                current_token.push(c);
            }
        }
    }

    if has_token {
        tokens.push(current_token);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::tokenize;
    #[test]
    fn test_simple() {
        assert_eq!(
            tokenize("echo").unwrap(),
            vec!["echo"]
        );
    }
    #[test]
    fn test_simple_tokens() {
        assert_eq!(
            tokenize("echo hello world").unwrap(),
            vec!["echo", "hello", "world"]
        );
    }

    #[test]
    fn test_double_quotes() {
        assert_eq!(
            tokenize(r#"echo "Hello, World!""#).unwrap(),
            vec!["echo", "Hello, World!"]
        );
    }

    #[test]
    fn test_unterminated_double_quote() {
        assert!(tokenize(r#"echo "unclosed"#).is_err());
    }

    #[test]
    fn test_single_quotes() {
        assert_eq!(
            tokenize("echo 'This is a test'").unwrap(),
            vec!["echo", "This is a test"]
        );
    }

    #[test]
    fn test_unterminated_single_quote() {
        assert!(tokenize("echo 'unclosed").is_err());
    }

    #[test]
    fn test_backslash_escape() {
        assert_eq!(
            tokenize(r"echo unquoted\ argument").unwrap(),
            vec!["echo", "unquoted argument"]
        );
    }

    #[test]
    fn test_mixed_quoting() {
        assert_eq!(
            tokenize(r#"echo "Hello, World!" 'This is a test' unquoted\ argument"#).unwrap(),
            vec!["echo", "Hello, World!", "This is a test", "unquoted argument"]
        );
    }

    #[test]
    fn test_empty_string_token() {
        assert_eq!(
            tokenize(r#"echo "" ''"#).unwrap(),
            vec!["echo", "", ""]
        );
    }

    #[test]
    fn test_escape_inside_double_quotes() {
        assert_eq!(
            tokenize(r#"echo "hello\"world""#).unwrap(),
            vec!["echo", r#"hello"world"#]
        );
        assert_eq!(
            tokenize(r#"e "hello\\world""#).unwrap(),
            vec!["e", r"hello\world"]
        );
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(tokenize("").unwrap(), Vec::<String>::new());
        assert_eq!(tokenize("   ").unwrap(), Vec::<String>::new());
    }

    #[test]
    fn test_adjacent_quoted_sections() {
        // 'hello'" world" should produce one token: "hello world"
        assert_eq!(
            tokenize(r#"echo 'hello'" world""#).unwrap(),
            vec!["echo", "hello world"]
        );
    }

    #[test]
    fn test_backslash_literal_in_double_quotes() {
        // \a is not a special escape, so backslash is kept literally
        assert_eq!(
            tokenize(r#"echo "hello\aworld""#).unwrap(),
            vec!["echo", r"hello\aworld"]
        );
    }

    // --- Bug-exposing tests ---

    #[test]
    fn test_backslash_quote_outside_quotes() {
        // Outside quotes: \" should produce a literal "
        // e.g.  echo hello\"world  → ["echo", "hello\"world"]
        assert_eq!(
            tokenize(r#"echo hello\"world"#).unwrap(),
            vec!["echo", r#"hello"world"#]
        );
    }

    #[test]
    fn test_single_quotes_protect_backslash() {
        // Inside single quotes, backslash is NOT special — it's literal
        assert_eq!(
            tokenize(r"echo 'hello\nworld'").unwrap(),
            vec!["echo", r"hello\nworld"]
        );
    }

    #[test]
    fn test_single_quotes_protect_double_quotes() {
        // Inside single quotes, double quote is literal
        assert_eq!(
            tokenize(r#"echo 'he said "hi"'"#).unwrap(),
            vec!["echo", r#"he said "hi""#]
        );
    }

    #[test]
    fn test_double_quotes_protect_single_quotes() {
        // Inside double quotes, single quote is literal
        assert_eq!(
            tokenize(r#"echo "it's fine""#).unwrap(),
            vec!["echo", "it's fine"]
        );
    }

    #[test]
    fn test_backslash_space_inside_double_quotes() {
        // Inside double quotes, \<space> is NOT a special escape,
        // so the backslash is literal
        assert_eq!(
            tokenize(r#"echo "hello\ world""#).unwrap(),
            vec!["echo", r"hello\ world"]
        );
    }

    #[test]
    fn test_trailing_backslash() {
        // A trailing backslash with nothing after it should be an error
        assert!(tokenize(r"echo hello\").is_err());
    }
}