use crate::builtin_functions::cd;
use crate::step::StepOutput;
/*
## Built-in Commands

* `cd <path>` makes the directory 'path' the current directory
* `exit` terminates foosh
* `quit` same as exit
* `fg [job-id]` sends job identified by jobid to foreground. If jobid is not specified, defaults to job which sate has been most recently modified.
* `bg [job-id]` sends job identified by jobid to background. If jobid is not specified, defaults to job which sate has been most recently modified.
* `jobs` output a list of currently active jobs  If a built-in command conflicts with the name of an external program, the built in command prevails --- unless the program path is explicitly given.
* `echo`
*/
use std::{fmt,io,io::{Error, ErrorKind}};

/// Args, Stdin
pub type BuiltinFn = fn(Vec<String>, &[u8]) -> StepOutput;

///Roughly analogous to process::Command
pub struct Builtin {
    name: String,
    args: Vec<String>,
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
            _ => Err(Error::new(ErrorKind::InvalidInput, "Empty Command")),
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

    ///Execute Logic
    /// Ok and result of the Builtin
    /// Err if the Builtin couldn't run
    pub fn run(self, stdin: &[u8]) -> io::Result<StepOutput> {
        let function = Builtin::function_map(&self.name)?;
        Ok((function)(self.args, stdin))
    }

    pub fn exists(name: &str) -> bool {
        Builtin::function_map(name).is_ok()
    }

}
    
mod test {
    use super::*;
    use std::{path::PathBuf,env};
    #[test]
    fn cd_root() {
        let b = Builtin::new("cd").arg("/");

        b.run(&[]);
        assert_eq!(env::current_dir().unwrap(),PathBuf::from("/"))
    }

    // #[test]
    //fn non_existing_builtin
}