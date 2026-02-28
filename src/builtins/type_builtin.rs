use crate::path_utils::find_in_path;

pub struct TypeBuiltin {
}

impl crate::builtins::Execute for TypeBuiltin {
    fn execute(&self, args: &[String], ctx: &mut crate::builtins::ShellContext) -> anyhow::Result<()> {
        if args.is_empty() {
            anyhow::bail!("Type args cannot be empty");
        }

        // Split args to get just the program name
        let program_name = args[0].as_str();

        if ctx.builtin_names.contains(&program_name) {
            println!("{} is a shell builtin", program_name)
        }
        else {
            match find_in_path(program_name, &ctx.path_dirs.iter().map(|s| s.as_str()).collect::<Vec<&str>>()) {
                Some(file_path) => println!("{} is {}", program_name, file_path.display()),
                None => println!("{}: not found", program_name),
            }
        }

        Ok(())
    }
}