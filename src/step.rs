use std::{
    io::{prelude::*, Error, ErrorKind, Result},
    process::{Command, Output, Stdio},
};

use crate::{redirection,builtin};

/// Step, the basic Unit of execution of a Pipeline. Can either be a Shrimp Built-in function or a Command
/// Design wise - a "Wrapper" enum was chosen because the Std::Command is a simple struct, it has no trait that builtins could implement (CommandExt are sealed)
#[derive(Debug)]
pub enum Step {
    Command(std::process::Command),
    Builtin(builtin::Builtin),
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
    /// Return the enum variant accordingly
    pub fn new(raw_step_str: &str) -> Result<Step> {
        //TODO Improve this step creation - CHECK IF BUILT-IN
        let c = Step::parse_command(raw_step_str)?;
        Ok(Step::Command(c))
    }

    /// Parses a string and returns a Command ready to be Executed, or and error.
    fn parse_command(raw_cmd_str: &str) -> Result<Command> {
        let cmd_string = String::from(raw_cmd_str);
        let mut words = cmd_string.split_whitespace();

        //Parse program
        let program = words.next();
        if program.is_none() {
            let e = Error::new(ErrorKind::InvalidInput, "Empty Program");
            return Err(e);
        }

        let mut command = Command::new(program.unwrap());

        for w in words {
            match w {
                //There can be no arguments after the beginning of redirection
                _ if redirection::Redirection::is_redirection(w) => {
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

    /// Runs the Step
    pub fn run(&mut self, stdin: &[u8]) -> Result<StepOutput> {
        match self {
            Step::Command(c) => {
                let mut process = c
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?;
                process.stdin.as_ref().unwrap().write_all(stdin)?;
                process.wait().unwrap();
                Ok(StepOutput::from(process.wait_with_output()?))
            }
            _ => unimplemented!(),
        }
    }
}
