use crate::builtin_functions::*;
use crate::step::StepOutput;
use std::{
    fmt, io,
    io::{Error, ErrorKind},
};

/// Built-in Function type, functions of this type implement the actual logic of the built-in commands in their respective files `cd`, `exit`, etc.Builtin
/// 
/// It takes as input a Vec for Args, and an array of Bytes as Stdin
pub type BuiltinFn = fn(Vec<String>, &[u8]) -> StepOutput;

///Roughly analogous to process::Command
pub struct Builtin {
    pub name: String,
    pub args: Vec<String>,
}

impl fmt::Debug for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builtin")
            .field("name", &self.name)
            .field("args", &self.args)
            .finish()
    }
}

impl Builtin {
    fn function_map(name: &str) -> io::Result<BuiltinFn> {
        match name {
            "cd" => Ok(cd::run),
            "exit" | "quit" => Ok(exit::run),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Non-existing Built-in")),
        }
    }

    pub fn new(name: &str) -> Builtin {
        Builtin {
            name: String::from(name),
            args: vec![],
        }
    }

    pub fn arg(mut self, new_arg: &str) -> Builtin {
        self.args.push(String::from(new_arg));
        self
    }

    ///Execute Logic, returning the StepOutput of the Builtin, or Err if it couldn run
    pub fn run(self, stdin: &[u8]) -> io::Result<StepOutput> {
        let function = Builtin::function_map(&self.name)?;
        Ok((function)(self.args, stdin))
    }

    ///Checks if a builtin of a given name exists in the the Builtin's function_map
    pub fn exists(name: &str) -> bool {
        Builtin::function_map(name).is_ok()
    }
}

mod test {
    use super::Builtin;
    #[test]
    fn cd_root() {
        use std::{env, path::PathBuf};

        let b = Builtin::new("cd").arg("/");

        let _r = b.run(&[]);
        assert_eq!(env::current_dir().unwrap(), PathBuf::from("/"))
    }

    #[test]
    fn non_existing_builtin() {
        let b = Builtin::new("oasijgoi").arg("3");

        let e = b.run(&[]);
        assert_eq!(e.is_err(), true)
    }
}
