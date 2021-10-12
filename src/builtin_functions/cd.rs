use crate::step::StepOutput;
use std::{env, path::PathBuf};

pub fn run(args: Vec<String>, _input: &[u8]) -> StepOutput {
    let mut code: Option<i32> = Some(0);
    let mut success = false;
    let stdout: Vec<u8> = vec![];
    let mut stderr: Vec<u8> = vec![];

    //Return to Home
    let path = if !args.is_empty() {
        PathBuf::from(&args[0])
    } else {
        PathBuf::from(&env::var("HOME").unwrap_or_default())
    };

    // if malformed_path
    if env::set_current_dir(path).is_err() {
        stderr.extend_from_slice("cd: Directory not found\n".as_bytes());
        success = false;
        code = Some(1);
    }

    // if cd failed
    StepOutput {
        success,
        code,
        stdout,
        stderr,
    }
}
