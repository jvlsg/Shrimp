use std::{
    collections::HashMap,
    fs::File,
    // io::Result,
    process::{Command, Stdio},
};

#[derive(Debug)]
pub struct Pipeline {
    commands: Vec<Command>,
}

impl Pipeline {
    // pub fn new(raw_string: &str) -> Self {
    //     let pipeline_str = String::from(raw_string);
    //     //TODO implement error piping
    //     //If ‘|&’ is used, command1’s standard error, in addition to its standard output, is connected to command2’s standard input through the pipe;
    //     let mut commands: Vec<String> = vec![];

    //     let split_vec: Vec<&str> = pipeline_str.split('|').collect();
    //     for s in split_vec {
    //         commands.push(String::from(s));
    //     }
    //     dbg!(&commands);
    //     Pipeline { commands }
    // }

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
pub fn parse_command(raw_cmd_string: &str) -> Result<Command, String> {
    let cmd_string = String::from(raw_cmd_string);
    let mut words = cmd_string.split_whitespace();

    //Parse program
    let program = words.next();
    if program.is_none() {
        return Err(String::from("Empty Program"));
    }

    let mut command = Command::new(program.unwrap());

    while let Some(w) = words.next() {
        match w {
            _ if redirections::is_redirection(w) => {
                let filename = words.next();
                if filename.is_none() {
                    return Err(String::from("Empty File redirection"));
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
    use std::fs::File;
    use std::process::{Command, Stdio};

    pub fn is_redirection(token: &str) -> bool {
        match token {
            "<" | ">" | "1>" | "2>" | ">>" | "&>" | "&>>" => true,
            _ => false,
        }
    }

    pub fn redirect(
        redirection: &str,
        filename: &str,
        command: &mut Command,
    ) -> Result<(), String> {
        match redirection {
            "<" => read_in(filename, command),
            ">" | "1>" => write_out(filename, command),
            "2>" => write_err(filename, command),
            ">>" => append_out(filename, command),
            "&>" => write_out_err(filename, command),
            "&>>" => append_out_err(filename, command),
            _ => panic!("Invalid redirection"),
        }
    }

    ///Sets stdin of the command as a file given by the filename
    fn read_in(filename: &str, command: &mut Command) -> Result<(), String> {
        if let Ok(file) = File::open(filename) {
            dbg!(&file);
            command.stdin(file);
            return Ok(());
        } else {
            Err(format!("Error Opening File {}", filename))
        }
    }

    fn write_out(filename: &str, command: &mut Command) -> Result<(), String> {
        if let Ok(file) = File::create(filename) {
            command.stdout(file);
            return Ok(());
        } else {
            Err(format!("Error Opening File {}", filename))
        }
    }

    fn write_err(filename: &str, command: &mut Command) -> Result<(), String> {
        if let Ok(file) = File::create(filename) {
            command.stderr(file);
            return Ok(());
        } else {
            Err(format!("Error Opening File {}", filename))
        }
    }

    //Write output and error
    fn write_out_err(filename: &str, command: &mut Command) -> Result<(), String> {
        if let Ok(file) = File::create(filename) {
            //TODO IMPROVE TRY_CLONE ERROR HANDLING
            command.stderr(file.try_clone().unwrap());
            command.stdout(file);
            return Ok(());
        } else {
            Err(format!("Error Opening File {}", filename))
        }
    }

    fn append_out(filename: &str, command: &mut Command) -> Result<(), String> {
        unimplemented!();
    }
    fn append_out_err(filename: &str, command: &mut Command) -> Result<(), String> {
        unimplemented!();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_simple_cmd() {
        let c = parse_command("ls");
        assert_eq!(c.is_ok(), true);
    }

    #[test]
    fn test_parse_simple_cmd_existing_input() {
        let c = parse_command("wc -c < tests/input > tests/output");
        assert_eq!(c.is_ok(), true);
    }
    #[test]
    fn test_parse_simple_cmd_empty_output() {
        let c = parse_command("wc -c < ");
        dbg!(&c);
        assert_eq!(c.is_ok(), false);
    }

    #[test]
    fn test_parse_simple_cmd_non_existing_input() {
        let c = parse_command("wc -c < tests/inputs > tests/output");
        assert_eq!(c.is_ok(), false);
    }

    #[test]
    fn test_simple_cmd_create_new_output() {
        let c = parse_command("ls -la < tests/input > tests/output_new");
        assert_eq!(c.is_ok(), true);
    }

    #[test]
    fn test_simple_cmd_output_err() {
        let c = parse_command("ping a 2> tests/err");
        assert_eq!(c.is_ok(), true);
    }

    fn test_simple_cmd_overwrite_output() {
        let c = parse_command("ls -la < tests/input > tests/output");
        assert_eq!(c.is_ok(), true);
    }

    fn test_simple_cmd_append_output() {
        let c = parse_command("ls -la < tests/input > tests/output");
        assert_eq!(c.is_ok(), true);
    }

    fn test_simple_cmd_redir_stderr() {
        let c = parse_command("ls -la < tests/input > tests/output");
        assert_eq!(c.is_ok(), true);
    }
}
