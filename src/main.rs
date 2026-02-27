#[allow(unused_imports)]
use std::env;
use std::io::{self, Write};
use std::process::Command;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(windows)]
const WINDOWS_EXECUTABLES: &[&str] = &["exe", "bat", "cmd", "com", "ps1"];

fn is_executable(file_path: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        if file_path.exists() {
            if let Ok(metadata) = std::fs::metadata(file_path) {
                let permissions = metadata.permissions();
                return permissions.mode() & 0o111 != 0;
            }
        }
        false
    }

    #[cfg(windows)]
    {
        if file_path.exists() {
            if let Some(ext) = file_path.extension() {
                let ext = ext.to_str().unwrap_or("").to_lowercase();
                return WINDOWS_EXECUTABLES.contains(&ext.as_str());
            }
        }
        false
    }
}

fn find_in_path(program_name: &str, path_dirs: &[&str]) -> Option<std::path::PathBuf> {
    for dir in path_dirs {
        let file_path = std::path::Path::new(dir).join(program_name);
        if is_executable(&file_path) {
            return Some(file_path);
        }
    }
    None
}

fn is_absolute_path(path: &str) -> bool {
    // Check for Unix absolute path (starts with /)
    if path.starts_with('/') {
        return true;
    }

    // Check for Windows absolute path (e.g., C:\, D:\)
    #[cfg(windows)]
    {
        if path.len() >= 3 {
            let chars: Vec<char> = path.chars().collect();
            if chars[0].is_alphabetic() && chars[1] == ':' && (chars[2] == '\\' || chars[2] == '/') {
                return true;
            }
        }
    }

    false
}

fn expand_tilde(path: &str) -> Result<String, String> {
    if path == "~" {
        // Just ~, return home directory
        env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())
    } else if path.starts_with("~/") {
        // ~/something, replace ~ with home directory
        match env::var("HOME") {
            Ok(home) => Ok(path.replacen("~", &home, 1)),
            Err(_) => Err("HOME environment variable not set".to_string()),
        }
    } else {
        // No tilde, return as-is
        Ok(path.to_string())
    }
}

fn main() {
    let builtin_commands = vec!["exit", "echo", "type", "pwd", "cd"];

    let path = env::var("PATH").unwrap_or_default();
    let path_dirs: Vec<&str> = path.split(if cfg!(windows) { ';' } else { ':' }).collect();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // wait for command input
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        let input = buffer.trim_end().to_owned();

        // evaluate
        let tokens = match tokenize(&input) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("rush: {}", e);
                continue;
            }
        };
        if tokens.is_empty() {
            continue;
        }
        let (command, args) = (tokens[0].as_str(), tokens[1..].to_vec());

        match command {
            "exit" => break,
            "type" => {
                // Split args to get just the program name
                let program_name = args[0].as_str();

                if builtin_commands.contains(&program_name) {
                    println!("{} is a shell builtin", program_name)
                }
                else {
                    match find_in_path(program_name, &path_dirs) {
                        Some(file_path) => println!("{} is {}", program_name, file_path.display()),
                        None => println!("{}: not found", program_name),
                    }
                }
            },
            "echo" => println!("{}", args.join(" ")),
            "pwd" => {
                match env::current_dir() {
                    Ok(path) => println!("{}", path.display()),
                    Err(e) => eprintln!("pwd: error getting current directory: {}", e),
                }
            },
            "cd" => {
                let target_dir = args.join(" ");

                if target_dir.is_empty() {
                    eprintln!("cd: missing argument");
                } else {
                    // Expand tilde if present
                    let expanded_path = match expand_tilde(&target_dir) {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("cd: {}", e);
                            continue;
                        }
                    };

                    // Determine the target path
                    let path = if is_absolute_path(&expanded_path) {
                        // Absolute path
                        Path::new(&expanded_path).to_path_buf()
                    } else {
                        // Relative path - resolve relative to current directory
                        match env::current_dir() {
                            Ok(current) => current.join(&expanded_path),
                            Err(e) => {
                                eprintln!("cd: error getting current directory: {}", e);
                                continue;
                            }
                        }
                    };

                    // Check if the path exists and is a directory
                    if path.exists() && path.is_dir() {
                        if let Err(e) = env::set_current_dir(&path) {
                            eprintln!("cd: {}: {}", target_dir, e);
                        }
                    } else {
                        println!("cd: {}: No such file or directory", target_dir);
                    }
                }
            },
            _ => {
                // Try to execute the command as an external program
                if find_in_path(command, &path_dirs).is_some() {
                    // Split args into individual arguments
                    let program_args: Vec<&str> = if args.is_empty() {
                        vec![]
                    } else {
                        args.iter().map(|s| s.as_str()).collect()
                    };

                    // Execute the command using just the command name
                    match Command::new(command).args(&program_args).output() {
                        Ok(output) => {
                            io::stdout().write_all(&output.stdout).unwrap();
                            io::stderr().write_all(&output.stderr).unwrap();
                        }
                        Err(e) => {
                            eprintln!("Failed to execute {}: {}", command, e);
                        }
                    }
                } else {
                    println!("{}: command not found", command);
                }
            }
        }

        io::stdout().flush().unwrap();
    }
}

fn tokenize(input: &str) -> Result<Vec<String>, String> {
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
                        None => return Err("Unterminated single quote".to_string()),
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
                        None => return Err("Unterminated double quote".to_string()),
                    }
                }
            }
            // --- Unquoted backslash: next char is literal ---
            '\\' => {
                has_token = true;
                match chars.next() {
                    Some(ch) => current_token.push(ch),
                    None => return Err("Trailing backslash".to_string()),
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
    use super::*;

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