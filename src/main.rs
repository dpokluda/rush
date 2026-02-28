mod tokenizer;
mod builtins;
mod path_utils;

use std::env;
use std::io::{self, Write};
use std::process::Command;
use tokenizer::tokenize;
use crate::builtins::{Builtin, Execute};
use crate::path_utils::find_in_path;

fn main() -> anyhow::Result<()> {
    let path = env::var("PATH").unwrap_or_default();
    let path_dirs = path.split(if cfg!(windows) { ';' } else { ':' }).map(|s| s.to_string()).collect();
    let mut ctx = builtins::ShellContext::new(path_dirs);

    loop {
        print!("$ ");
        io::stdout().flush()?;

        // wait for command input
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
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

        // if exit, break
        if command == "exit" {
            break Ok(());
        }

        match Builtin::from_name(command){
            Some(builtin) => {
                if let Err(e) = builtin.execute(&args, &mut ctx) {
                    eprintln!("rush: {}", e);
                }
            },
            None => {
                // Try to execute as an external program
                let path_dirs_ref: Vec<&str> = ctx.path_dirs.iter().map(|s| s.as_str()).collect();
                if find_in_path(command, &path_dirs_ref).is_some() {
                    let program_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                    match Command::new(command).args(&program_args).output() {
                        Ok(output) => {
                            io::stdout().write_all(&output.stdout)?;
                            io::stderr().write_all(&output.stderr)?;
                        }
                        Err(e) => {
                            eprintln!("rush: failed to execute {}: {}", command, e);
                        }
                    }
                } else {
                    eprintln!("{}: command not found", command);
                }
            },
        }

        io::stdout().flush()?;
    }
}