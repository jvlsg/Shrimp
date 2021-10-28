use std::{
    io::{prelude::*, Error, ErrorKind, Result},
    process::{Command, Output, Stdio},
};

use crate::{builtin::Builtin, redirection};

/// Step, the basic Unit of execution of a Pipeline. Can either be a Shrimp Built-in function or a Command
/// Design wise - a "Wrapper" enum was chosen because the Std::Command is a simple struct, it has no trait that builtins could implement (CommandExt are sealed)
#[derive(Debug)]
pub enum Step {
    Command(std::process::Command),
    Builtin(Builtin),
}

/// Roughly analogous to process::Output mixed with process::ExitStatus.
/// Since process::ExitStatus is sealed, we can't instantiate it directly. With our own struct, Builtins can use it as well
#[derive(Debug)]
pub struct StepOutput {
    pub success: bool,
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl From<Output> for StepOutput {
    fn from(output: Output) -> StepOutput {
        Self {
            stdout: output.stdout,
            stderr: output.stderr,
            code: output.status.code(),
            success: output.status.success(),
        }
    }
}

impl Step {
    ///Creates a new Step. It will validate if the desired command is a Built-in or an external program and
    /// Return the enum variant accordingly.
    ///
    /// In the (extremely) unlikely scenario of naming conflict with a Built-in, the Built-in will take prescedence
    pub fn new(step_words: Vec<String>) -> Result<Step> {
        let mut words = step_words.into_iter().peekable();

        if words.peek().is_none() {
            let e = Error::new(ErrorKind::InvalidInput, "Empty Step");
            return Err(e);
        }

        //Check if builtin with that name exists
        if Builtin::exists(words.peek().unwrap()) {
            let b = Step::parse_builtin(words)?;
            Ok(Step::Builtin(b))
        } else {
            let c = Step::parse_command(words)?;
            Ok(Step::Command(c))
        }
    }

    /// Parses a peekable SplitWhitespace iterator and returns a Command ready to be Executed, or an error.
    /// Panics - If no values present in iterator - as this should be handled by the caller function, e.g. `Step::new`
    fn parse_command(
        mut words: std::iter::Peekable<std::vec::IntoIter<std::string::String>>,
    ) -> Result<Command> {
        let mut command = Command::new(words.next().unwrap());

        for w in words {
            match w {
                //There can be no arguments after the beginning of redirection
                _ if redirection::Redirection::is_redirection(&w) => {
                    break;
                }
                //Arguments
                _ => {
                    command.arg(&w);
                }
            }
        }
        Ok(command)
    }

    /// Parses a peekable SplitWhitespace iterator and returns a Builtin ready to be Executed, or an error.
    /// Panics - If no values present in iterator - as this should be handled by the caller function, e.g. `Step::new`
    fn parse_builtin(
        mut words: std::iter::Peekable<std::vec::IntoIter<std::string::String>>,
    ) -> Result<Builtin> {
        let mut b_in = Builtin::new(&words.next().unwrap());

        for w in words {
            match w {
                //There can be no arguments after the beginning of redirection
                _ if crate::redirection::Redirection::is_redirection(&w) => {
                    break;
                }
                //Arguments
                _ => {
                    b_in = b_in.arg(&w);
                }
            }
        }
        Ok(b_in)
    }

    /// Runs the Step
    /// Err if the Step couldn't run
    pub fn run(self, stdin: &[u8]) -> Result<StepOutput> {
        match self {
            Step::Command(mut c) => {
                let mut process = c
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?;
                process.stdin.as_ref().unwrap().write_all(stdin)?;
                process.wait().unwrap();
                Ok(StepOutput::from(process.wait_with_output()?))
            }
            Step::Builtin(b) => b.run(stdin),
        }
    }
}

mod test {
    use super::*;
    #[test]
    fn empty_step() {
        let s_str = vec![];
        let s = Step::new(s_str);
        assert_eq!(s.is_err(), true);
        assert_eq!(s.unwrap_err().kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn parse_simple_builtin() {
        let cd_str = vec![String::from("cd /home/user")];
        let b = Step::new(cd_str).unwrap();
        if let Step::Builtin(broa) = b {
            assert_eq!(&broa.name, "cd");
            assert_eq!(&broa.args, &vec![String::from("/home/user")]);
        }
    }
}
