use crate::{step::StepOutput};
use std::{env,path::Path};

pub fn run(args: Vec<String>,_input: &[u8]) -> StepOutput {

    let mut code: Option<i32> = None;
    let mut success = false;
    let mut stdout = false;
    let mut stderr = false;


    //if no args[0]
    let path = Path::new(&args[0]);

    // if malformed_path

    env::set_current_dir(path).unwrap();

    // if cd failed
    StepOutput{
        code:Some(0),
        stderr:vec!(),
        stdout:vec!(),
        success: true,
    }
}