use std::{
    io::{Error, ErrorKind, Result},
    process::{ChildStdout, Command, ExitStatus, Output, Stdio},
};

pub mod redirections;

mod builtins {
    #[derive(Debug)]
    ///Temporarily set as a Struct. Might change to a Trait in the future
    pub struct Builtin;
}

#[derive(Debug)]
/// Step, the basic Unit of execution of a Pipeline. Can either be a Shrimp Built-in function or a Command
/// Design wise - a "Wrapper" enum was chosen because the Std::Command is a simple struct, it has no trait that builtins could implement (CommandExt are sealed)
/// It will implement an API based on the functions of Std::Command
pub enum Step {
    Command(std::process::Command),
    Builtin(crate::builtins::Builtin),
}

impl Step {
    ///Creates a new Step. It will validate if the desired command is a Built-in or an external program and
    /// Return the enum variant accordingly
    pub fn new(raw_step_str: &str) -> Result<Step> {
        //TODO Improve this step creation - CHECK IF BUILT-IN
        //Return error if Empty String
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

        while let Some(w) = words.next() {
            match w {
                _ if redirections::is_redirection(w) => {
                    let filename = words.next();
                    if filename.is_none() {
                        let e = Error::new(
                            ErrorKind::InvalidInput,
                            String::from("Empty File redirection"),
                        );
                        return Err(e);
                    }
                    let filename = filename.unwrap();
                    redirections::redirect(w, filename, &mut command)?;
                }
                //Arguments
                _ => {
                    command.arg(&w);
                }
            }
        }
        // dbg!(&command);
        Ok(command)
    }

    ///Runs the Step, retuning the contents of Stdout
    pub fn get_output(&mut self) -> Result<Stdio> {
        match self {
            Step::Command(c) => {
                let mut child = c.stdout(Stdio::piped()).spawn()?;
                Ok(Stdio::from(child.stdout.take().unwrap()))
            }
            _ => unimplemented!(),
        }
    }

    ///Runs the Step, retuning the contents of EITHER Stdout and Stderr, depending on the process' exit code
    pub fn get_output_err(&mut self) -> Result<Stdio> {
        match self {
            Step::Command(c) => {
                let mut child = c
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap();
                
                let exit = child.wait()?;
                if exit.success(){
                    return Ok(Stdio::from(child.stdout.take().unwrap()));
                }
                return Ok(Stdio::from(child.stderr.take().unwrap()));
            }
            _ => unimplemented!(),
        }
    }

    //Reconfigures the step's Input
    pub fn set_input(&mut self, input: Stdio) {
        match self {
            Step::Command(c) => {
                c.stdin(input);
            }
            _ => unimplemented!(),
        }
    }

    ///Runs the step with it's pre-existing configuration / IO
    pub fn run(&mut self) -> Result<ExitStatus> {
        match self {
            Step::Command(c) => c.spawn().unwrap().wait(),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Pipe {
    Std,
    Err,
}

#[derive(Debug)]
///A pipeline is composed by Steps (commands or builtins), and Pipes that connect the output from one Stpe to the next
pub struct Pipeline {
    steps: Vec<Step>,
    pipes: Vec<Pipe>,
}

impl Pipeline {
    pub fn new(raw_pipeline_str: &str) -> Result<Self> {
        let mut pipeline_string = String::from(raw_pipeline_str);

        let mut steps: Vec<Step> = vec![];
        let mut pipes: Vec<Pipe> = vec![];

        loop {
            //Next pipe index - could be standard or Error
            let next_pipe = pipeline_string.find("|");

            //No / No more pipes, add remaining of the string to steps
            //This guarantees at least one step to the Pipeline
            if next_pipe.is_none() {
                steps.push(Step::new(&pipeline_string)?);
                break;
            }

            let next_pipe = next_pipe.unwrap();

            let next_err_pipe = pipeline_string.find("|&");

            //Check if the next pipe is standard , or error
            let is_err_pipe = next_pipe == next_err_pipe.unwrap_or(usize::MAX);

            let (step_str, remainder) = pipeline_string.split_at(next_pipe);
            steps.push(Step::new(&step_str)?);

            if is_err_pipe {
                pipes.push(Pipe::Err);
                pipeline_string = String::from(remainder.strip_prefix("|&").unwrap());
            } else {
                pipes.push(Pipe::Std);
                pipeline_string = String::from(remainder.strip_prefix("|").unwrap());
            }
        }

        dbg!(&steps);
        Ok(Pipeline { steps, pipes })
    }

    ///Executes all Steps, piping outputs/errors into inputs, 
    /// consuming the Pipeline
    pub fn run(self) -> Result<()> {

        //We know that all pipelines will have at least one Step
        //First step will run with the pre-configured input
        let mut step_iter = self.steps.into_iter();

        let mut curr_step = step_iter.next().unwrap();        
        
        //For each pipe, we redirect output / err according to pipe type
        for pipe in self.pipes.into_iter(){                
            dbg!(&curr_step);

            let next_input = match pipe {
                // we want to collect ONLY stdout
                Pipe::Std => curr_step.get_output()?,
                // We want to collect both stderr and stdout
                Pipe::Err => curr_step.get_output_err()?,
            };
            dbg!(&next_input);
            //Get next step and Set it's Input
            curr_step = step_iter.next().unwrap();
            curr_step.set_input(next_input);
        }

        dbg!(&curr_step);
        curr_step.run()?;
        return Ok(());
    }    
}


#[cfg(test)]
mod test {
    mod pipelines {
        use super::super::*;

        #[test]
        fn test_simple_pipeline() {
            let p = Pipeline::new("echo \"asd\" |& grep a | wc -c").unwrap();
            dbg!(&p.pipes);
            dbg!(&p.steps);
            assert_eq!(p.pipes, vec![Pipe::Err, Pipe::Std]);
        }

        #[test]
        fn test_empty_pipe() {
            let p = Pipeline::new("");
            assert_eq!(p.is_err(), true);
            dbg!(&p);
        }

        #[test]
        fn test_empty_command() {
            let p = Pipeline::new("|& ls");
            assert_eq!(p.is_err(), true);
            dbg!(&p);
        }

        #[test]
        fn test_empty_command_2() {
            let p = Pipeline::new("echo \"asd\" grep a |& | wc -c");
            assert_eq!(p.is_err(), true);
            dbg!(&p);
        }
    }
}
