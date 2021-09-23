use crate::{redirection::Redirection, Step, StepOutput};
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
/// TODO: 2021-09-12 Change these two Enums?
pub trait PipelineWriter: std::io::Write + std::fmt::Debug {}
impl PipelineWriter for File {}
impl PipelineWriter for Stdout {}
impl PipelineWriter for Stderr {}

///A pipeline is composed by Steps (commands or builtins), and Pipes that connect the output from one Step to the next
pub struct Pipeline {
    steps: Vec<Step>,
    pipes: Vec<Pipe>,
    in_reader: Option<Box<dyn PipelineReader>>,
    out_writer: Box<dyn PipelineWriter>,
    err_writer: Box<dyn PipelineWriter>,
    redirection_write_type: Option<Redirection>, //Used to change write logic
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

        let mut redirection_write_type = None;

        let mut words = pipeline_string.split_whitespace();

        //Find redirection
        while let Some(w) = words.next() {
            if let Ok(redir) = Redirection::from_str(w) {
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

                if redir != Redirection::ReadIn {
                    redirection_write_type = Some(redir);
                }
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

        Ok(Pipeline {
            pipes,
            steps,
            in_reader,
            out_writer: out_writer.unwrap_or(Box::new(std::io::stdout())),
            err_writer: err_writer.unwrap_or(Box::new(std::io::stderr())),
            redirection_write_type,
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

        //Appends Output INTO Err, write both to the same destination
        //If we simply wrote each to it's destination, in case of `&>`
        //Output would overrwrite Err.
        if self.redirection_write_type == Some(Redirection::WriteOutErr)
            || self.redirection_write_type == Some(Redirection::AppendOutErr)
        {
            last_out.stderr.append(&mut last_out.stdout);
            self.out_writer.write_all(&last_out.stderr)?;
        } else {
            self.err_writer.write_all(&last_out.stderr)?;
            self.out_writer.write_all(&last_out.stdout)?;
        }

        Ok(last_out)
    }
}

//********************/
#[cfg(test)]
mod test {
    use super::*;
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
            redirection_write_type: None,
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
            redirection_write_type: None,
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
            redirection_write_type: None,
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
            redirection_write_type: None,
        };

        let r = p.run().unwrap();
        assert_eq!(r.success, true);
    }

    #[test]
    fn parse_simple_pipeline() {
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
            redirection_write_type: None,
        };
        assert_eq!(p.pipes, p_str.pipes);
    }

    #[test]
    fn parse_pipeline_new_output() {
        let p_str = Pipeline::new("echo -n abcde | tr -d a | wc -c > tests/output_new").unwrap();
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
            redirection_write_type: None,
        };

        assert_eq!(p.pipes, p_str.pipes);

        let bla = format!("{:?}", p.out_writer);
        let bla: Vec<&str> = bla.split(",").collect();

        let lab = format!("{:?}", p_str.out_writer);
        let lab: Vec<&str> = lab.split(",").collect();

        assert_eq!(bla[1..], lab[1..]);
    }

    #[test]
    fn parse_empty_pipeline() {
        let p = Pipeline::new("");
        assert_eq!(p.is_err(), true);
        dbg!(&p);
    }

    #[test]
    fn parse_non_existing_input() {
        let c = Pipeline::new("wc -c < tests/inputs > tests/output");
        assert!(c.is_err());
        assert_eq!(c.unwrap_err().kind(), ErrorKind::NotFound);
    }

    #[test]
    fn simple_pipeline_read_existing_write_output_existing_file() {
        Pipeline::new("wc -c < tests/lorem > tests/output")
            .unwrap()
            .run()
            .unwrap();

        let mut buff = String::new();
        let mut file = File::open("tests/output").unwrap();
        file.read_to_string(&mut buff).unwrap();
        assert_eq!("447", buff.trim());
    }

    #[test]
    fn simple_pipeline_overwrite_output_file() {
        Pipeline::new("wc -c < tests/lorem > tests/output")
            .unwrap()
            .run()
            .unwrap();
        Pipeline::new("wc -w < tests/lorem > tests/output")
            .unwrap()
            .run()
            .unwrap();

        let mut buff = String::new();
        let mut file = File::open("tests/output").unwrap();
        file.read_to_string(&mut buff).unwrap();
        assert_eq!("69", buff.trim());
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
    fn pipeline_write_output_create_new_file() {
        let _p = Pipeline::new("echo -n abcde | tr -d a | wc -c > tests/output_new")
            .unwrap()
            .run()
            .unwrap();

        let mut buff = String::new();
        let mut file = File::open("tests/output_new").unwrap();
        file.read_to_string(&mut buff).unwrap();
        assert_eq!("4", buff.trim());
        fs::remove_file("tests/output_new").unwrap();
    }

    #[test]
    fn pipeline_append_output_existing_file() {
        Pipeline::new("echo test > tests/output")
            .unwrap()
            .run()
            .unwrap();

        let _p = Pipeline::new("echo -n abcde | tr -d a | wc -c >> tests/output")
            .unwrap()
            .run()
            .unwrap();

        let mut buff = String::new();
        let mut file = File::open("tests/output").unwrap();
        file.read_to_string(&mut buff).unwrap();
        assert_eq!("test\n4", buff.trim());
    }

    #[test]
    fn pipeline_append_err_existing_file() {
        Pipeline::new("echo test > tests/output")
            .unwrap()
            .run()
            .unwrap();

        let _p = Pipeline::new("echo -n abcde | tr -d a | wc -x 2>> tests/output")
            .unwrap()
            .run()
            .unwrap();

        let mut buff = String::new();
        let mut file = File::open("tests/output").unwrap();
        file.read_to_string(&mut buff).unwrap();
        assert_eq!(
            "test\nwc: invalid option -- 'x'\nTry 'wc --help' for more information.",
            buff.trim()
        );
    }
    #[test]
    fn pipeline_write_error_existing_file() {
        let res = Pipeline::new("ping a 2> tests/err").unwrap().run().unwrap();
        assert_eq!(res.success, false);
        assert_ne!(res.code, Some(0));

        let mut buff = String::new();
        let mut file = File::open("tests/err").unwrap();
        file.read_to_string(&mut buff).unwrap();
        assert_eq!("ping: a: Name or service not known", buff.trim());
    }

    #[test]
    fn pipeline_write_output_and_error_existing_file() {
        let res = Pipeline::new("ls tests/err erro &> tests/output")
            .unwrap()
            .run()
            .unwrap();
        assert_eq!(res.success, false);
        assert_ne!(res.code, Some(0));

        let mut buff = String::new();
        let mut file = File::open("tests/output").unwrap();
        file.read_to_string(&mut buff).unwrap();
        assert_eq!(
            "ls: cannot access 'erro': No such file or directory\ntests/err",
            buff.trim()
        );
    }

    #[test]
    fn pipeline_write_output_and_error_existing_file_alt() {
        let res = Pipeline::new("ls tests/err erro 2>&1 tests/output")
            .unwrap()
            .run()
            .unwrap();
        assert_eq!(res.success, false);
        assert_ne!(res.code, Some(0));

        let mut buff = String::new();
        let mut file = File::open("tests/output").unwrap();
        file.read_to_string(&mut buff).unwrap();
        assert_eq!(
            "ls: cannot access 'erro': No such file or directory\ntests/err",
            buff.trim()
        );
    }

    #[test]
    fn pipeline_append_output_and_error_existing_file() {
        Pipeline::new("echo test > tests/output")
            .unwrap()
            .run()
            .unwrap();

        let res = Pipeline::new("ls tests/err erro &>> tests/output")
            .unwrap()
            .run()
            .unwrap();
        assert_eq!(res.success, false);
        assert_ne!(res.code, Some(0));

        let mut buff = String::new();
        let mut file = File::open("tests/output").unwrap();
        file.read_to_string(&mut buff).unwrap();
        assert_eq!(
            "test\nls: cannot access 'erro': No such file or directory\ntests/err",
            buff.trim()
        );
    }
}
