///Module with functions to Handle IO Redirections
use std::{
    fs::{File, OpenOptions},
    io::{prelude::*, Result},
    process::Command,
    str::FromStr,
};

pub enum Redirection {
    ReadIn,
    WriteOut,
    AppendOut,
    WriteErr,
    AppendErr,
    WriteOutErr,
    AppendOutErr,
}

pub struct RedirectionParseError {}

impl FromStr for Redirection {
    type Err = RedirectionParseError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "<" => Ok(Redirection::ReadIn),
            ">" | "1>" => Ok(Redirection::WriteOut),
            ">>" => Ok(Redirection::WriteOut),
            "2>" => Ok(Redirection::WriteOut),
            "2>>" => Ok(Redirection::WriteOut),
            "&>" | "2>&1" => Ok(Redirection::WriteOut),
            "&>>" => Ok(Redirection::WriteOut),
            _ => Err(RedirectionParseError {}),
        }
    }
}

impl Redirection {
    pub fn is_redirection(s: &str) -> bool{

        if let Ok(_) = Redirection::from_str(s){
            return true;
        }

        false
    }

    pub fn redirect(
        &self,
        filename: &str,
        in_reader: &mut Option<Box<dyn Read>>,
        out_writer: &mut Option<Box<dyn Write>>,
        err_writer: &mut Option<Box<dyn Write>>,
    ) -> () {
        match self{
            Redirection::ReadIn => {

            }
            Redirection::WriteOut => {

            }
            Redirection::AppendOut => {

            }
            Redirection::WriteErr => {

            }
            Redirection::AppendErr => {

            }
            Redirection::WriteOutErr => {

            }
            Redirection::AppendOutErr => {

            }
        }
    }
}

// Possibly
// pub fn redirect(redirection: &str, filename: &str, &in, &out, &err) -> Result<()> {
pub fn redirect(redirection: &str, filename: &str) -> Result<()> {
    match redirection {
        "<" => read_in(filename),
        ">" | "1>" => write_out(filename),
        ">>" => append_out(filename),
        "2>" => write_err(filename),
        "2>>" => append_err(filename),
        "&>" | "2>&1" => write_out_err(filename),
        "&>>" => append_out_err(filename),
        _ => panic!("Invalid redirection"),
    }
}

//Sets stdin of the command as a file given by the filename
fn read_in(filename: &str) -> Result<()> {
    let file = File::open(filename)?;
    // command.stdin(file);
    Ok(())
}

fn write_out(filename: &str) -> Result<()> {
    let file = File::create(filename)?;
    // command.stdout(file);
    Ok(())
}

fn write_err(filename: &str) -> Result<()> {
    let file = File::create(filename)?;
    // command.stderr(file);
    Ok(())
}

//Write output and error
fn write_out_err(filename: &str) -> Result<()> {
    let file = File::create(filename)?;
    // command.stderr(file.try_clone().unwrap());
    // command.stdout(file);
    Ok(())
}

fn append_out(filename: &str) -> Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)?;
    // command.stdout(file);
    Ok(())
}

fn append_err(filename: &str) -> Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)?;
    // command.stderr(file);
    Ok(())
}

fn append_out_err(filename: &str) -> Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)?;
    // command.stderr(file.try_clone().unwrap());
    // command.stdout(file);
    Ok(())
}

mod test {
    use crate::Step;
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
