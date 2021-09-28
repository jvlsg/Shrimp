use crate::step::StepOutput;
use std::{env, path::PathBuf};

pub fn run(args: Vec<String>, _input: &[u8]) -> StepOutput {
    let mut code: Option<i32> = Some(0);
    let mut success = false;
    let stdout: Vec<u8> = vec![];
    let mut stderr: Vec<u8> = vec![];

    //Return to Home
    let path = if args.len() != 0 {
        PathBuf::from(&args[0])
    } else {
        PathBuf::from(&env::var("HOME").unwrap_or_default())
    };
    
    
    // if malformed_path
    if let Err(_) = env::set_current_dir(path){
        stderr.extend_from_slice("Directory not found".as_bytes());
        success = false;
        code = Some(1);
    }
    
    // if cd failed
    StepOutput {
        code,
        stderr,
        stdout,
        success,
    }
}
