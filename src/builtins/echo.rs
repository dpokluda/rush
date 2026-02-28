use crate::builtins::Execute;

pub struct EchoBuiltin {
}

impl Execute for EchoBuiltin {
    fn execute(&self, args: &[String], _ctx: &mut crate::builtins::ShellContext) -> anyhow::Result<()> {
        println!("{}", args.join(" "));
        Ok(())
    }
}