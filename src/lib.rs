use std::process::Command;

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
    use std::fs::{File, OpenOptions};
    use std::process::{Command, Stdio};

    pub fn is_redirection(token: &str) -> bool {
        matches!(token, "<" | ">" | "1>" | "2>" | ">>" | "&>" | "&>>")
    }

    pub fn redirect(
        redirection: &str,
        filename: &str,
        command: &mut Command,
    ) -> Result<(), String> {
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
    fn read_in(filename: &str, command: &mut Command) -> Result<(), String> {
        if let Ok(file) = File::open(filename) {
            dbg!(&file);
            command.stdin(file);
            Ok(())
        } else {
            Err(format!("Error Opening File {}", filename))
        }
    }

    fn write_out(filename: &str, command: &mut Command) -> Result<(), String> {
        if let Ok(file) = File::create(filename) {
            command.stdout(file);
            Ok(())
        } else {
            Err(format!("Error Opening File {}", filename))
        }
    }

    fn write_err(filename: &str, command: &mut Command) -> Result<(), String> {
        if let Ok(file) = File::create(filename) {
            command.stderr(file);
            Ok(())
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
            Ok(())
        } else {
            Err(format!("Error Opening File {}", filename))
        }
    }

    fn append_out(filename: &str, command: &mut Command) -> Result<(), String> {
        if let Ok(file) = OpenOptions::new().append(true).create(true).open(filename) {
            command.stdout(file);
            Ok(())
        } else {
            Err(format!("Error Opening File {}", filename))
        }
    }

    fn append_err(filename: &str, command: &mut Command) -> Result<(), String> {
        if let Ok(file) = OpenOptions::new().append(true).create(true).open(filename) {
            command.stderr(file);
            Ok(())
        } else {
            Err(format!("Error Opening File {}", filename))
        }
    }

    fn append_out_err(filename: &str, command: &mut Command) -> Result<(), String> {
        if let Ok(file) = OpenOptions::new().append(true).create(true).open(filename) {
            command.stderr(file.try_clone().unwrap());
            command.stdout(file);
            Ok(())
        } else {
            Err(format!("Error Opening File {}", filename))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
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
