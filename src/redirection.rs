///Module with functions to Handle IO Redirections
use std::{
    fs::{File, OpenOptions},
    io::{prelude::*, Result},
    net::{SocketAddr, ToSocketAddrs},
    str::FromStr,
};

use crate::pipeline::{PipelineReader, PipelineWriter};

pub enum Redirection {
    ReadIn,
    WriteOut,
    AppendOut,
    WriteErr,
    AppendErr,
    WriteOutErr,
    AppendOutErr,
}

#[derive(Debug)]
pub struct RedirectionParseError {}

impl FromStr for Redirection {
    type Err = RedirectionParseError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "<" => Ok(Redirection::ReadIn),
            ">" | "1>" => Ok(Redirection::WriteOut),
            ">>" => Ok(Redirection::AppendOut),
            "2>" => Ok(Redirection::WriteErr),
            "2>>" => Ok(Redirection::AppendErr),
            "&>" | "2>&1" => Ok(Redirection::WriteOutErr),
            "&>>" => Ok(Redirection::AppendOutErr),
            _ => Err(RedirectionParseError {}),
        }
    }
}

impl Redirection {
    pub fn is_redirection(s: &str) -> bool {
        Redirection::from_str(s).is_ok()
    }

    /// Gets the `src_or_dst` and mutable references to the Readers and Writers
    /// Depending on the type of redirection and the type Reader/Writer of `src_or_dst`
    /// it updates the mut references accordingly.
    pub fn configure_redirection(
        &self,
        src_or_dst: &str,
        in_reader: &mut Option<Box<dyn PipelineReader>>,
        out_writer: &mut Option<Box<dyn PipelineWriter>>,
        err_writer: &mut Option<Box<dyn PipelineWriter>>,
    ) -> Result<()> {
        //src_or_dst is a Socket
        if let Ok(_socket) = src_or_dst.to_socket_addrs() {
            //TODO 2021-08-28 Implement for network
            //NOT SURE THIS WORKS FOR URLS
            // match self {
            //     Redirection::ReadIn => {}
            //     Redirection::WriteOut => {}
            //     Redirection::AppendOut => {}
            //     Redirection::WriteErr => {}
            //     Redirection::AppendErr => {}
            //     Redirection::WriteOutErr => {}
            //     Redirection::AppendOutErr => {}
            // }
        }
        //Default to a path in the filesystem
        else {
            //change redir
            match self {
                Redirection::ReadIn => {
                    //Set contents of the reference as
                    *in_reader = Some(Box::new(File::open(src_or_dst)?));
                }
                Redirection::WriteOut => {
                    *out_writer = Some(Box::new(File::create(src_or_dst)?));
                }
                Redirection::AppendOut => {
                    *out_writer = Some(Box::new(
                        OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(src_or_dst)?,
                    ));
                }
                Redirection::WriteErr => {
                    *err_writer = Some(Box::new(File::create(src_or_dst)?));
                }
                Redirection::AppendErr => {
                    *err_writer = Some(Box::new(
                        OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(src_or_dst)?,
                    ));
                }
                Redirection::WriteOutErr => {
                    *out_writer = Some(Box::new(File::create(src_or_dst)?));
                    *err_writer = Some(Box::new(File::create(src_or_dst)?));
                }
                Redirection::AppendOutErr => {
                    *out_writer = Some(Box::new(
                        OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(src_or_dst)?,
                    ));

                    *err_writer = Some(Box::new(
                        OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(src_or_dst)?,
                    ));
                }
            }
            return Ok(());
        }

        Ok(())
    }
}
mod test {

    // #[test]
    // fn test_simple_cmd_create_new_output() {
    //     {
    //         let c = Step::parse_command("wc -c < tests/lorem > tests/output_new");
    //         assert_eq!(c.is_ok(), true);

    //         c.unwrap().spawn().unwrap().wait().unwrap();

    //         let mut buff = String::new();
    //         let mut file = File::open("tests/output_new").unwrap();
    //         file.read_to_string(&mut buff).unwrap();
    //         assert_eq!("447", buff.trim());
    //     }
    //     fs::remove_file("tests/output_new").unwrap();
    // }

    // #[test]
    // fn test_simple_cmd_output_err() {
    //     let c = Step::parse_command("ping a 2> tests/err");
    //     assert_eq!(c.is_ok(), true);

    //     c.unwrap().spawn().unwrap().wait().unwrap();

    //     let mut buff = String::new();
    //     let mut file = File::open("tests/err").unwrap();
    //     file.read_to_string(&mut buff).unwrap();
    //     assert_eq!("ping: a: Name or service not known", buff.trim());
    // }

    // #[test]
    // fn test_simple_cmd_overwrite_output() {
    //     let c = Step::parse_command("wc -c < tests/lorem > tests/output");
    //     c.unwrap().spawn().unwrap().wait().unwrap();

    //     let c = Step::parse_command("wc -w < tests/lorem > tests/output");
    //     c.unwrap().spawn().unwrap().wait().unwrap();

    //     let mut buff = String::new();
    //     let mut file = File::open("tests/output").unwrap();
    //     file.read_to_string(&mut buff).unwrap();
    //     assert_eq!("69", buff.trim());
    // }

    // #[test]
    // fn test_simple_cmd_append_output() {
    //     let c = Step::parse_command("wc -c < tests/lorem > tests/output");
    //     assert_eq!(c.is_ok(), true);

    //     c.unwrap().spawn().unwrap().wait().unwrap();

    //     let mut buff = String::new();
    //     let mut file = File::open("tests/output").unwrap();
    //     file.read_to_string(&mut buff).unwrap();
    //     assert_eq!("447", buff.trim());

    //     let c = Step::parse_command("wc -w < tests/lorem >> tests/output");
    //     assert_eq!(c.is_ok(), true);

    //     c.unwrap().spawn().unwrap().wait().unwrap();

    //     let mut buff = String::new();
    //     let mut file = File::open("tests/output").unwrap();
    //     file.read_to_string(&mut buff).unwrap();
    //     assert_eq!("447\n69", buff.trim());
    // }

    // #[test]
    // fn test_simple_cmd_redir_stderr() {
    //     let c = Step::parse_command("wc -x 2> tests/output");
    //     assert_eq!(c.is_ok(), true);

    //     c.unwrap().spawn().unwrap().wait().unwrap();

    //     let mut buff = String::new();
    //     let mut file = File::open("tests/output").unwrap();
    //     file.read_to_string(&mut buff).unwrap();
    //     assert!(buff.trim().starts_with("wc: invalid option -- 'x'"));
    // }
}
