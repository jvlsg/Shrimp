use std::{
    io::{Error, ErrorKind, Result},
    process::{Command, Output, ExitStatus, Stdio},
};

mod builtins {
    #[derive(Debug)]
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
    pub fn new(raw_step_str: &str) -> Result<Step> {
        //TODO Improve this step creation - CHECK IF BUILT-IN
        //Return error if Empty String
        let c = Step::parse_command(raw_step_str)?;
        Ok(Step::Command(c))
    }

    /// Parses a string and returns a Command ready to be Executed, or and error.
    pub fn parse_command(raw_cmd_str: &str) -> Result<Command> {
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
        dbg!(&command);
        Ok(command)
    }

    // pub fn stdout(&mut self, out: Stdio){
    //     match self {
    //         Step::Command(mut c) =>{
    //             // c.stdout(out);
    //             self = Step::Builtin;
    //         },
    //         _ => unimplemented!(),
    //     }
    // }

    pub fn output(&mut self) -> Result<Output> {
        match self {
            Step::Command(c) => {
                c.output()
            },
            _ => unimplemented!(),
        }
    }

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

    ///Executes all Steps, piping outputs/errors into inputs, consuming the steps and pipes
    pub fn run(&mut self) -> Result<()> {

        //We know that all pipelines will have at least one Step
        let mut curr_step = self.steps.pop().unwrap();

        //Run current step, pipe
        while !self.steps.is_empty(){
            
            let last_output = curr_step.output();
            //Get next step
            curr_step = self.steps.pop().unwrap();
            
            // match self.pipes.pop().unwrap(){
            //     Std => {
            //         curr_step.stdout()
            //     },
            //     Err => {

            //     }
            // }
        }

        curr_step.run()?;
        return Ok(());
    }
}

mod redirections {
    use std::{
        fs::{File, OpenOptions},
        io::Result,
        process::Command,
    };

    pub fn is_redirection(token: &str) -> bool {
        matches!(token, "<" | ">" | "1>" | "2>" | ">>" | "&>" | "&>>")
    }

    pub fn redirect(redirection: &str, filename: &str, command: &mut Command) -> Result<()> {
        match redirection {
            "<" => read_in(filename, command),
            ">" | "1>" => write_out(filename, command),
            ">>" => append_out(filename, command),
            "2>" => write_err(filename, command),
            "2>>" => append_err(filename, command),
            "&>" | "2>&1" => write_out_err(filename, command),
            "&>>" => append_out_err(filename, command),
            _ => panic!("Invalid redirection"),
        }
    }

    //Sets stdin of the command as a file given by the filename
    fn read_in(filename: &str, command: &mut Command) -> Result<()> {
        let file = File::open(filename)?;
        command.stdin(file);
        Ok(())
    }

    fn write_out(filename: &str, command: &mut Command) -> Result<()> {
        let file = File::create(filename)?;
        command.stdout(file);
        Ok(())
    }

    fn write_err(filename: &str, command: &mut Command) -> Result<()> {
        let file = File::create(filename)?;
        command.stderr(file);
        Ok(())
    }

    //Write output and error
    fn write_out_err(filename: &str, command: &mut Command) -> Result<()> {
        let file = File::create(filename)?;
        command.stderr(file.try_clone().unwrap());
        command.stdout(file);
        Ok(())
    }

    fn append_out(filename: &str, command: &mut Command) -> Result<()> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(filename)?;
        command.stdout(file);
        Ok(())
    }

    fn append_err(filename: &str, command: &mut Command) -> Result<()> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(filename)?;
        command.stderr(file);
        Ok(())
    }

    fn append_out_err(filename: &str, command: &mut Command) -> Result<()> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(filename)?;
        command.stderr(file.try_clone().unwrap());
        command.stdout(file);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    mod redirections {
        use super::super::*;
        use std::fs::{self, File};
        use std::io::prelude::*;

        #[test]
        fn test_parse_simple_cmd() {
            let c = Step::parse_command("ls");
            assert_eq!(c.is_ok(), true);
        }

        #[test]
        fn test_parse_simple_cmd_io_redirections() {
            let c = Step::parse_command("wc -c < tests/lorem > tests/output");
            assert_eq!(c.is_ok(), true);

            c.unwrap().spawn().unwrap().wait().unwrap();

            let mut buff = String::new();
            let mut file = File::open("tests/output").unwrap();
            file.read_to_string(&mut buff).unwrap();
            assert_eq!("447", buff.trim());
        }

        #[test]
        fn test_parse_simple_cmd_empty_output() {
            let c = Step::parse_command("wc -c < ");
            dbg!(&c);
            assert!(c.is_err())
        }

        #[test]
        fn test_parse_simple_cmd_non_existing_input() {
            let c = Step::parse_command("wc -c < tests/inputs > tests/output");
            assert!(c.is_err())
        }

        #[test]
        fn test_simple_cmd_create_new_output() {
            {
                let c = Step::parse_command("wc -c < tests/lorem > tests/output_new");
                assert_eq!(c.is_ok(), true);

                c.unwrap().spawn().unwrap().wait().unwrap();

                let mut buff = String::new();
                let mut file = File::open("tests/output_new").unwrap();
                file.read_to_string(&mut buff).unwrap();
                assert_eq!("447", buff.trim());
            }
            fs::remove_file("tests/output_new").unwrap();
        }

        #[test]
        fn test_simple_cmd_output_err() {
            let c = Step::parse_command("ping a 2> tests/err");
            assert_eq!(c.is_ok(), true);

            c.unwrap().spawn().unwrap().wait().unwrap();

            let mut buff = String::new();
            let mut file = File::open("tests/err").unwrap();
            file.read_to_string(&mut buff).unwrap();
            assert_eq!("ping: a: Name or service not known", buff.trim());
        }

        #[test]
        fn test_simple_cmd_overwrite_output() {
            let c = Step::parse_command("wc -c < tests/lorem > tests/output");
            c.unwrap().spawn().unwrap().wait().unwrap();

            let c = Step::parse_command("wc -w < tests/lorem > tests/output");
            c.unwrap().spawn().unwrap().wait().unwrap();

            let mut buff = String::new();
            let mut file = File::open("tests/output").unwrap();
            file.read_to_string(&mut buff).unwrap();
            assert_eq!("69", buff.trim());
        }

        #[test]
        fn test_simple_cmd_append_output() {
            let c = Step::parse_command("wc -c < tests/lorem > tests/output");
            assert_eq!(c.is_ok(), true);

            c.unwrap().spawn().unwrap().wait().unwrap();

            let mut buff = String::new();
            let mut file = File::open("tests/output").unwrap();
            file.read_to_string(&mut buff).unwrap();
            assert_eq!("447", buff.trim());

            let c = Step::parse_command("wc -w < tests/lorem >> tests/output");
            assert_eq!(c.is_ok(), true);

            c.unwrap().spawn().unwrap().wait().unwrap();

            let mut buff = String::new();
            let mut file = File::open("tests/output").unwrap();
            file.read_to_string(&mut buff).unwrap();
            assert_eq!("447\n69", buff.trim());
        }

        #[test]
        fn test_simple_cmd_redir_stderr() {
            let c = Step::parse_command("wc -x 2> tests/output");
            assert_eq!(c.is_ok(), true);

            c.unwrap().spawn().unwrap().wait().unwrap();

            let mut buff = String::new();
            let mut file = File::open("tests/output").unwrap();
            file.read_to_string(&mut buff).unwrap();
            assert!(buff.trim().starts_with("wc: invalid option -- 'x'"));
        }
    }

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
