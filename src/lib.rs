use std::{
    fmt,
    io::{prelude::*, Error, ErrorKind, Result},
    process::{Command, Output, Stdio},
    str::FromStr,
};

pub mod redirection;

mod builtins {
    #[derive(Debug)]
    pub struct Builtin;
    ///Roughly analogous to process::Command
    impl Builtin {
        pub fn new() {}
        //Possibly implement from String to do the parsing, similar to Step::parse_command
    }
}

/// Step, the basic Unit of execution of a Pipeline. Can either be a Shrimp Built-in function or a Command
/// Design wise - a "Wrapper" enum was chosen because the Std::Command is a simple struct, it has no trait that builtins could implement (CommandExt are sealed)
#[derive(Debug)]
pub enum Step {
    Command(std::process::Command),
    Builtin(crate::builtins::Builtin),
}

///Roughly analogous to process::Output mixed with process::ExitStatus.
/// Since process::ExitStatus is sealed, we can't instantiate it directly. With our own struct, Builtins can use it as well
#[derive(Debug)]
pub struct StepOutput {
    success: bool,
    code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
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

    ///Runs the Step
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

#[derive(Debug, PartialEq)]
pub enum Pipe {
    Std,
    Err,
}

///A pipeline is composed by Steps (commands or builtins), and Pipes that connect the output from one Stpe to the next
/// TODO 2021-07-09 Change trait objects to Generics?
pub struct Pipeline {
    steps: Vec<Step>,
    pipes: Vec<Pipe>,
    in_reader: Option<Box<dyn Read>>,
    out_writer: Box<dyn Write>,
    err_writer: Box<dyn Write>,
}

impl fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pipeline")
            .field("Steps", &self.steps)
            .field("Pipes", &self.pipes)
            .finish()
    }
}

impl Pipeline {
    /// TODO 2021-07-21 Currently the parsing implementation is naive, improve it.
    pub fn new(raw_pipeline_str: &str) -> Result<Pipeline> {
        let mut pipeline_string = String::from(raw_pipeline_str);

        let mut steps: Vec<Step> = vec![];
        let mut pipes: Vec<Pipe> = vec![];

        let in_reader: Option<Box<dyn Read>> = None;
        let out_writer: Option<Box<dyn Write>> = None;
        let err_writer: Option<Box<dyn Write>> = None;

        let mut words = pipeline_string.split_whitespace();

        //Find redirection
        while let Some(w) = words.next() {
            if let Ok(redir) = redirection::Redirection::from_str(w) {
                //TODO 2021-08-02 GET REDIRECTION TYPE, SET TO CORRECT VARIABLE
                //Pipeline only needs the Type of redirection (so it knows which variable to set) and the corresponding
                //reader / writer

                let src_or_dst = words.next();
                if src_or_dst.is_none() {
                    return Err(Error::new(ErrorKind::InvalidInput, "Empty redirection"));
                }
                let src_or_dst = src_or_dst.unwrap();

                //We'll only be able to set the reader / writer (File, Socket, etc) DEPENDING on the redirection Type
                //
                /*
                match redir.get()
                    RedirWrite =>
                */
            }
        }

        //Find Pipes & Steps
        loop {
            //Next pipe index - could be standard or Error
            let next_pipe = pipeline_string.find('|');

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

        //     dbg!(&steps);
        Ok(Pipeline {
            pipes,
            steps,
            in_reader,
            out_writer: out_writer.unwrap_or(Box::new(std::io::stdout())),
            err_writer: err_writer.unwrap_or(Box::new(std::io::stderr())),
        })
    }

    ///Executes all Steps, piping outputs/errors into inputs,
    /// consuming the Pipeline and returning the `StepOutput` of the last step
    pub fn run(mut self) -> Result<StepOutput> {
        let mut pipeline_input = Vec::new();

        //Read pipeline input from input source, if any
        if let Some(mut src) = self.in_reader {
            src.read_to_end(&mut pipeline_input)?;
        }

        let mut step_iter = self.steps.into_iter();

        let curr_step = step_iter.next();

        if curr_step.is_none() {
            return Err(Error::new(ErrorKind::InvalidInput, "No Steps on Pipeline"));
        }

        let mut curr_step = curr_step.unwrap();

        dbg!(&curr_step);
        dbg!(String::from_utf8_lossy(&pipeline_input));

        let mut last_out = curr_step.run(&pipeline_input)?;

        //For each pipe, we redirect output / err according to pipe type
        for pipe in self.pipes.into_iter() {
            let curr_input = match pipe {
                // we want to collect ONLY stdout
                Pipe::Std => last_out.stdout,
                // We want to collect both stderr and stdout
                Pipe::Err => {
                    last_out.stdout.append(&mut last_out.stderr);
                    last_out.stdout
                }
            };

            dbg!(String::from_utf8_lossy(&curr_input));

            curr_step = step_iter.next().unwrap();

            dbg!(&curr_step);

            last_out = curr_step.run(&curr_input)?;
        }

        dbg!(&last_out.code);
        dbg!(&last_out.success);

        self.out_writer.write_all(&last_out.stdout)?;
        self.err_writer.write_all(&last_out.stderr)?;

        Ok(last_out)
    }
}

#[cfg(test)]
mod test {
    mod pipelines {
        use super::super::*;
        use std::fs::File;

        #[test]
        fn simple_pipeline() {
            let p = Pipeline {
                steps: vec![
                    Step::new("echo -n abcde").unwrap(),
                    Step::new("tr -d a").unwrap(),
                    Step::new("wc -c").unwrap(),
                ],
                pipes: vec![Pipe::Std, Pipe::Std],
                in_reader: None,
                out_writer: Box::new(std::io::stdout()),
                err_writer: Box::new(std::io::stderr()),
            };

            let r = p.run().unwrap();
            assert_eq!(r.success, true);
        }

        #[test]
        fn simple_pipeline_redir_in() {
            let p = Pipeline {
                steps: vec![Step::new("wc -c").unwrap()],
                pipes: vec![],
                in_reader: Some(Box::new(File::open("tests/lorem").unwrap())),
                out_writer: Box::new(std::io::stdout()),
                err_writer: Box::new(std::io::stderr()),
            };

            let r = p.run().unwrap();
            assert_eq!(r.success, true);
        }

        #[test]
        fn empty_pipeline() {
            let p = Pipeline {
                steps: vec![],
                pipes: vec![],
                in_reader: None,
                out_writer: Box::new(std::io::stdout()),
                err_writer: Box::new(std::io::stderr()),
            };

            let r = p.run();
            assert_eq!(r.is_err(), true);
        }

        #[test]
        fn simple_pipeline_err() {
            let p = Pipeline {
                steps: vec![
                    Step::new("echo -n abcde").unwrap(),
                    Step::new("tr -3 a").unwrap(),
                    Step::new("wc -c").unwrap(),
                ],
                pipes: vec![Pipe::Err, Pipe::Err],
                in_reader: None,
                out_writer: Box::new(std::io::stdout()),
                err_writer: Box::new(std::io::stderr()),
            };

            let r = p.run().unwrap();
            assert_eq!(r.success, true);
        }

        #[test]
        fn simple_pipeline_parsing() {
            let p = Pipeline::new("echo \"asd\" |& grep a | wc -c").unwrap();
            dbg!(&p);
            assert_eq!(p.pipes, vec![Pipe::Err, Pipe::Std]);
        }

        #[test]
        fn empty_pipeline_parsing() {
            let p = Pipeline::new("");
            assert_eq!(p.is_err(), true);
            dbg!(&p);
        }

        //     #[test]
        //     fn test_empty_command() {
        //         let p = Pipeline::new("|& ls");
        //         assert_eq!(p.is_err(), true);
        //         dbg!(&p);
        //     }

        //     #[test]
        //     fn test_empty_command_2() {
        //         let p = Pipeline::new("echo \"asd\" grep a |& | wc -c");
        //         assert_eq!(p.is_err(), true);
        //         dbg!(&p);
        //     }
    }
    #[test]
    fn bytes_test() {
        use std::io::prelude::*;
        use std::process::{Command, Stdio};
        use std::str::FromStr;

        static PANGRAM: &'static str = "the quick brown fox jumped over the lazy dog\n";

        // Spawn the `wc` command
        let mut process = match Command::new("wc")
            .arg("-c")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
        {
            Err(why) => panic!("couldn't spawn wc: {}", why),
            Ok(process) => process,
        };

        // Write a string to the `stdin` of `wc`.
        //
        // `stdin` has type `Option<ChildStdin>`, but since we know this instance
        // must have one, we can directly `unwrap` it.
        match process
            .stdin
            .as_ref()
            .unwrap()
            .write_all(PANGRAM.as_bytes())
        {
            Err(why) => panic!("couldn't write to wc stdin: {}", why),
            Ok(_) => println!("sent pangram to wc"),
        }
        process.wait().unwrap();
        // Because `stdin` does not live after the above calls, it is `drop`ed,
        // and the pipe is closed.
        //
        // This is very important, otherwise `wc` wouldn't start processing the
        // input we just sent.

        // The `stdout` field also has type `Option<ChildStdout>` so must be unwrapped.
        let mut s = String::new();
        match process.stdout.unwrap().read_to_string(&mut s) {
            Err(why) => panic!("couldn't read wc stdout: {}", why),
            Ok(_) => print!("wc responded with:\n{}", &s),
        }

        // ---
        // let out = process.wait_with_output().unwrap();
        // let mut s = String::from_utf8_lossy(&out.stdout);

        let x = i32::from_str(&s.trim()).unwrap();
        println!("{}", x + 1);
    }
}
