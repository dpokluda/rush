use crate::builtins::cd::CdBuiltin;
use crate::builtins::echo::EchoBuiltin;
use crate::builtins::pwd::PwdBuiltin;
use crate::builtins::type_builtin::TypeBuiltin;

mod echo;
mod pwd;
mod type_builtin;
mod cd;

pub enum Builtin {
    Echo(EchoBuiltin),
    Cd(CdBuiltin),
    Pwd(PwdBuiltin),
    Type(TypeBuiltin),
}

impl Execute for Builtin {
    fn execute(&self, args: &[String], ctx: &mut ShellContext) -> anyhow::Result<()> {
        match self {
            Builtin::Echo(b) => b.execute(args, ctx),
            Builtin::Cd(b) => b.execute(args, ctx),
            Builtin::Pwd(b) => b.execute(args, ctx),
            Builtin::Type(b) => b.execute(args, ctx),
        }
    }
}

impl Builtin {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "echo" => Some(Builtin::Echo(EchoBuiltin {})),
            "cd" => Some(Builtin::Cd(CdBuiltin {})),
            "pwd" => Some(Builtin::Pwd(PwdBuiltin {})),
            "type" => Some(Builtin::Type(TypeBuiltin {})),
            _ => None,
        }
    }
}

const BUILTINS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];

pub struct ShellContext{
    pub path_dirs: Vec<String>,
    pub builtin_names: Vec<&'static str>,
}

impl ShellContext {
    pub fn new(path_dirs: Vec<String>) -> Self {
        ShellContext {
            path_dirs,
            builtin_names: BUILTINS.to_vec(),
        }
    }
}

pub trait Execute {
    fn execute(&self, args: &[String], ctx: &mut ShellContext) -> anyhow::Result<()>;
}