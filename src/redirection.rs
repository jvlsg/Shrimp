///Module with functions to Handle IO Redirections
use std::{
    fs::{File, OpenOptions},
    io::Result,
    net::{SocketAddr, ToSocketAddrs},
    str::FromStr,
};

use crate::pipeline::{PipelineReader, PipelineWriter};

#[derive(Debug, std::cmp::PartialEq)]
pub enum Redirection {
    ReadIn,
    WriteOut,
    AppendOut,
    WriteErr,
    AppendErr,
    WriteOutErr,
    AppendOutErr,
}

#[derive(Debug, std::cmp::PartialEq)]
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
        if let Ok(socket) = src_or_dst.to_socket_addrs() {
            //TODO 2021-08-28 Implement for network
            //NOT SURE THIS WORKS FOR URLS
            match self {
                Redirection::ReadIn => {}
                Redirection::WriteOut => {}
                Redirection::AppendOut => {}
                Redirection::WriteErr => {}
                Redirection::AppendErr => {}
                Redirection::WriteOutErr => {}
                Redirection::AppendOutErr => {}
            }
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
                //These last two only have out_writer set, as it will be the sole destination
                //for both output and err
                Redirection::WriteOutErr => {
                    *out_writer = Some(Box::new(File::create(src_or_dst)?));
                }
                Redirection::AppendOutErr => {
                    *out_writer = Some(Box::new(
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
    use super::*;
    #[test]
    fn parse_redirection() {
        assert_eq!(Redirection::from_str("<"), Ok(Redirection::ReadIn));
        assert_eq!(Redirection::from_str(">"), Ok(Redirection::WriteOut));
        assert_eq!(Redirection::from_str(">>"), Ok(Redirection::AppendOut));
        assert_eq!(Redirection::from_str("2>"), Ok(Redirection::WriteErr));
        assert_eq!(Redirection::from_str("2>>"), Ok(Redirection::AppendErr));
        assert_eq!(Redirection::from_str("&>"), Ok(Redirection::WriteOutErr));
        assert_eq!(Redirection::from_str("2>&1"), Ok(Redirection::WriteOutErr));
        assert_eq!(Redirection::from_str("&>>"), Ok(Redirection::AppendOutErr));
    }
}