use crate::{redirection, Step, StepOutput};
use std::{
    fmt,
    fs::File,
    io::{prelude::*, Error, ErrorKind, Result, Stderr, Stdin, Stdout},
    str::FromStr,
};

#[derive(Debug, PartialEq)]
pub enum Pipe {
    Std,
    Err,
}

///These combine additional traits, such as Debug, to the Readers used by the Pipeline
pub trait PipelineReader: std::io::Read + std::fmt::Debug {}
impl PipelineReader for File {}
impl PipelineReader for Stdin {}

///These combine additional traits, such as Debug, to the Writers used by the Pipeline
pub trait PipelineWriter: std::io::Write + std::fmt::Debug {}
impl PipelineWriter for File {}
impl PipelineWriter for Stdout {}
impl PipelineWriter for Stderr {}

///A pipeline is composed by Steps (commands or builtins), and Pipes that connect the output from one Stpe to the next
/// TODO 2021-07-09 Change trait objects to Generics?
pub struct Pipeline {
    steps: Vec<Step>,
    pipes: Vec<Pipe>,
    in_reader: Option<Box<dyn PipelineReader>>,
    out_writer: Box<dyn PipelineWriter>,
    err_writer: Box<dyn PipelineWriter>,
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

        let mut in_reader: Option<Box<dyn PipelineReader>> = None;
        let mut out_writer: Option<Box<dyn PipelineWriter>> = None;
        let mut err_writer: Option<Box<dyn PipelineWriter>> = None;

        let mut words = pipeline_string.split_whitespace();

        //Find redirection
        while let Some(w) = words.next() {
            if let Ok(redir) = redirection::Redirection::from_str(w) {
                //Pipeline only needs the Type of redirection (so it knows which variable to set) and the corresponding
                //reader / writer

                let src_or_dst = words.next();
                if src_or_dst.is_none() {
                    return Err(Error::new(ErrorKind::InvalidInput, "Empty redirection"));
                }
                let src_or_dst = src_or_dst.unwrap();

                //We'll only be able to set the reader / writer (File, Socket, etc) DEPENDING on the redirection Type
                redir.configure_redirection(
                    src_or_dst,
                    &mut in_reader,
                    &mut out_writer,
                    &mut err_writer,
                )?;
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

//******************* */
#[cfg(test)]
mod test {
    mod pipelines {
        use super::super::*;
        use std::fs::{self, File};

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
            let p_str = Pipeline::new("echo \"asd\" |& grep a | wc -c").unwrap();
            dbg!(&p_str);

            let p = Pipeline {
                steps: vec![
                    Step::new("echo \"asd\"").unwrap(),
                    Step::new("grep a").unwrap(),
                    Step::new("wc -c").unwrap(),
                ],
                pipes: vec![Pipe::Err, Pipe::Std],
                in_reader: None,
                out_writer: Box::new(std::io::stdout()),
                err_writer: Box::new(std::io::stderr()),
            };
            assert_eq!(p.pipes, p_str.pipes);
        }

        #[test]
        fn pipeline_parsing_new_output() {
            let p_str =
                Pipeline::new("echo -n abcde | tr -d a | wc -c > tests/output_new").unwrap();
            let p = Pipeline {
                steps: vec![
                    Step::new("echo -n abcde").unwrap(),
                    Step::new("tr -d a").unwrap(),
                    Step::new("wc -c").unwrap(),
                ],
                pipes: vec![Pipe::Std, Pipe::Std],
                in_reader: None,
                out_writer: Box::new(File::create("tests/output_new").unwrap()),
                err_writer: Box::new(std::io::stderr()),
            };
            
            assert_eq!(p.pipes, p_str.pipes);

            let bla = format!("{:?}",p.out_writer);
            let bla: Vec<&str> = bla.split(",").collect();

            let lab = format!("{:?}",p_str.out_writer);
            let lab: Vec<&str> = lab.split(",").collect();

            assert_eq!(bla[1..],lab[1..]);
        }

        #[test]
        fn empty_pipeline_parsing() {
            let p = Pipeline::new("");
            assert_eq!(p.is_err(), true);
            dbg!(&p);
        }

        #[test]
        fn simple_pipeline_read_existing_write_out_existing() {
            let p_str = Pipeline::new("wc -c < tests/lorem > tests/output").unwrap();
            dbg!(p_str.run());

            let mut buff = String::new();
            let mut file = File::open("tests/output").unwrap();
            file.read_to_string(&mut buff).unwrap();
            assert_eq!("447", buff.trim());
        }

        #[test]
        fn simple_pipeline_empty_out_redir() {
            let p = Pipeline::new("wc -c < ");
            assert_eq!(p.is_err(), true);
            assert_eq!(p.unwrap_err().kind(), ErrorKind::InvalidInput);
        }

        #[test]
        fn three_step_pipeline() {
            let p_res = Pipeline::new("echo -n abcde | tr -d a | wc -c")
                .unwrap()
                .run()
                .unwrap();
            dbg!(&p_res);
            assert_eq!(String::from_utf8(p_res.stdout).unwrap().trim(), "4")
        }

        #[test]
        fn three_step_pipeline_create_new_output_file() {
            {
                //Stuff still bugging out here
                let p = Pipeline::new("echo -n abcde | tr -d a | wc -c > tests/output_new")
                    .unwrap();
                
                println!("a");
                dbg!(&p.out_writer);

                let p_res = p                    
                    .run()
                    .unwrap();
                // dbg!(&p_res);
                // let mut buff = String::new();
                // let mut file = File::open("tests/output_new").unwrap();
                // file.read_to_string(&mut buff).unwrap();
                // assert_eq!("4", buff.trim());
            }
            // fs::remove_file("tests/output_new").unwrap();
        }
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
