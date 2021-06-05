use std::{
    cmp,
    io::{Error, ErrorKind, Result},
    process::Command,
};

mod builtins {
    #[derive(Debug)]
    pub struct Builtin;
}

#[derive(Debug)]
pub enum Step {
    Command(std::process::Command),
    Builtin(crate::builtins::Builtin),
}

impl Step {
    pub fn new_step(raw_step_str: &str) -> Result<Step> {
        //TODO Improve this step creation - CHECK IF BUILT-IN
        //Return error if Empty String
        let c = parse_command(raw_step_str)?;
        Ok(Step::Command(c))
    }
}

#[derive(Debug)]
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
            dbg!(next_pipe);            

            //No more pipes, add remaining of the string to steps
            if next_pipe.is_none() {
                steps.push(Step::new_step(&pipeline_string)?);
                break;
            }

            let next_pipe = next_pipe.unwrap();

            let next_err_pipe = pipeline_string.find("|&");
            dbg!(next_err_pipe);

            //Check if the next pipe is standard , or error
            let is_err_pipe = next_pipe == next_err_pipe.unwrap_or(usize::MAX);
                        
            let (step_str, remainder) = pipeline_string.split_at(next_pipe);
            steps.push(Step::new_step(&step_str)?);

            dbg!(is_err_pipe);
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

    // pub fn run(&self) -> () {
    //     let mut c_iter = self.commands.iter();

    //     //1st
    //     let mut last_output =
    //         run_command(c_iter.next().unwrap(), None, Some(Stdio::piped()), None).unwrap();

    //     dbg!(&last_output);

    //     while let Some(c) = c_iter.next(){
    //         dbg!(&c);
    //         last_output = run_command(
    //             c,
    //             Some(Stdio::from(last_output)),
    //             None,
    //             None,
    //         )
    //         .unwrap()
    //     }

    //     ()
    // }
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
            let c = parse_command("ls");
            assert_eq!(c.is_ok(), true);
        }

        #[test]
        fn test_parse_simple_cmd_io_redirections() {
            let c = parse_command("wc -c < tests/lorem > tests/output");
            assert_eq!(c.is_ok(), true);

            c.unwrap().spawn().unwrap().wait().unwrap();

            let mut buff = String::new();
            let mut file = File::open("tests/output").unwrap();
            file.read_to_string(&mut buff).unwrap();
            assert_eq!("447", buff.trim());
        }

        #[test]
        fn test_parse_simple_cmd_empty_output() {
            let c = parse_command("wc -c < ");
            dbg!(&c);
            assert!(c.is_err())
        }

        #[test]
        fn test_parse_simple_cmd_non_existing_input() {
            let c = parse_command("wc -c < tests/inputs > tests/output");
            assert!(c.is_err())
        }

        #[test]
        fn test_simple_cmd_create_new_output() {
            {
                let c = parse_command("wc -c < tests/lorem > tests/output_new");
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
            let c = parse_command("ping a 2> tests/err");
            assert_eq!(c.is_ok(), true);

            c.unwrap().spawn().unwrap().wait().unwrap();

            let mut buff = String::new();
            let mut file = File::open("tests/err").unwrap();
            file.read_to_string(&mut buff).unwrap();
            assert_eq!("ping: a: Name or service not known", buff.trim());
        }

        #[test]
        fn test_simple_cmd_overwrite_output() {
            let c = parse_command("wc -c < tests/lorem > tests/output");
            c.unwrap().spawn().unwrap().wait().unwrap();

            let c = parse_command("wc -w < tests/lorem > tests/output");
            c.unwrap().spawn().unwrap().wait().unwrap();

            let mut buff = String::new();
            let mut file = File::open("tests/output").unwrap();
            file.read_to_string(&mut buff).unwrap();
            assert_eq!("69", buff.trim());
        }

        #[test]
        fn test_simple_cmd_append_output() {
            let c = parse_command("wc -c < tests/lorem > tests/output");
            assert_eq!(c.is_ok(), true);

            c.unwrap().spawn().unwrap().wait().unwrap();

            let mut buff = String::new();
            let mut file = File::open("tests/output").unwrap();
            file.read_to_string(&mut buff).unwrap();
            assert_eq!("447", buff.trim());

            let c = parse_command("wc -w < tests/lorem >> tests/output");
            assert_eq!(c.is_ok(), true);

            c.unwrap().spawn().unwrap().wait().unwrap();

            let mut buff = String::new();
            let mut file = File::open("tests/output").unwrap();
            file.read_to_string(&mut buff).unwrap();
            assert_eq!("447\n69", buff.trim());
        }

        #[test]
        fn test_simple_cmd_redir_stderr() {
            let c = parse_command("wc -x 2> tests/output");
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
        // use std::fs::{self, File};
        // use std::io::prelude::*;

        #[test]
        fn simple_pipeline() {
            let p = Pipeline::new("echo \"asd\" | grep a ").unwrap();
            dbg!(p.pipes);
            dbg!(p.steps);
        }

        #[test]
        fn simple_pipe_err() {
            let p = Pipeline::new("wc -l |& grep e").unwrap();
            dbg!(p.pipes);
            dbg!(p.steps);
        }        
    }
}
