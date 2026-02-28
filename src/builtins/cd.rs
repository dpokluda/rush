use std::env;
use std::path::Path;
use crate::path_utils::{expand_tilde, is_absolute_path};

pub struct CdBuiltin {
}

impl crate::builtins::Execute for CdBuiltin {
    fn execute(&self, args: &[String], _ctx: &mut crate::builtins::ShellContext) -> anyhow::Result<()> {
        let home_dir = &"~".to_string();

        let target_dir = if args.is_empty() {
            home_dir
        } else {
            &args[0]
        };

        // expand tilde if present
        let expanded_path = match expand_tilde(target_dir) {
            Ok(path) => path,
            Err(e) => anyhow::bail!("cd: {}", e),
        };

         // Determine the target path
        let path = if is_absolute_path(&expanded_path) {
            // Absolute path
            Path::new(&expanded_path).to_path_buf()
        } else {
            // Relative path - resolve relative to current directory
            match env::current_dir() {
                Ok(current) => current.join(&expanded_path),
                Err(e) => anyhow::bail!("cd: error getting current directory: {}", e),
            }
         };

        // Check if the path exists and is a directory
        if path.exists() && path.is_dir() {
            if let Err(e) = env::set_current_dir(&path) {
                anyhow::bail!("cd: {}: {}", target_dir, e)
            }
        } else {
            anyhow::bail!("cd: {}: No such file or directory", target_dir)
        }

        Ok(())
    }
}