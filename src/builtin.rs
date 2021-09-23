use crate::step::StepOutput;
use crate::builtins::cd;
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
use std::{fmt,collections::hash_map};

/// Args, Stdin
pub type BuiltinFn = fn(Vec<String>,&[u8]) -> StepOutput;


pub struct Builtin{
    name: String,
    args: Vec<String>,
    function: BuiltinFn,
}

impl fmt::Debug for Builtin{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Point")
         .field("name", &self.name)
         .field("args", &self.args)
         .finish()
    }
}

///Roughly analogous to process::Command
impl Builtin {
    fn function_map(name: &str) -> Result<BuiltinFn,()>{
        match name{
            "cd" => Ok(cd::run),
            _ => Err(())
        }
    }

    pub fn new(name: &str) -> Builtin{
        //Dispatch table that overrides the run function? I.e. the builtins are just a function
        Builtin{
            name: String::from(name),
            args: vec!(),
            function: Builtin::function_map(name).unwrap(),
        }
    }
    
    pub fn arg(mut self,new_arg: &str){
        self.args.push(String::from(new_arg));
    }

    //Possibly implement from String to do the parsing, similar to Step::parse_command

    pub fn run(self, stdin: &[u8]) -> StepOutput {
        (self.function)(self.args,stdin)
    }

    pub fn exists(name: &str) -> bool{
        //Dispatch table that overrides the run function? I.e. the builtins are just a function
        Builtin::function_map(name).is_ok()
    }

}
