use std::env;

use crate::builtins::Execute;

pub struct PwdBuiltin {
}

impl Execute for PwdBuiltin {
    fn execute(&self, _args: &[String], _ctx: &mut crate::builtins::ShellContext) -> anyhow::Result<()> {
        match env::current_dir() {
            Ok(path) => println!("{}", path.display()),
            Err(e) => eprintln!("pwd: error getting current directory: {}", e),
        }
        Ok(())
    }
}