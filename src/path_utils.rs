use std::env;
use anyhow::Context;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(windows)]
const WINDOWS_EXECUTABLES: &[&str] = &["exe", "bat", "cmd", "com", "ps1"];

pub fn is_executable(file_path: &std::path::Path) -> bool {
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

pub fn find_in_path(program_name: &str, path_dirs: &[&str]) -> Option<std::path::PathBuf> {
    for dir in path_dirs {
        let file_path = std::path::Path::new(dir).join(program_name);
        if is_executable(&file_path) {
            return Some(file_path);
        }
    }
    None
}

pub fn is_absolute_path(path: &str) -> bool {
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

pub fn expand_tilde(path: &str) -> anyhow::Result<String> {
    if path == "~" {
        // Just ~, return home directory
        env::var("HOME").context("HOME environment variable not set")
    } else if path.starts_with("~/") {
        // ~/something, replace ~ with home directory
        let home = env::var("HOME").context("HOME environment variable not set")?;
        Ok(path.replacen("~", &home, 1))
    } else {
        // No tilde, return as-is
        Ok(path.to_string())
    }
}