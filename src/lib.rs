use std::{
    // io::Result,
    process::{Command, Stdio},
    fs::File,
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
pub fn parse_command(
    raw_cmd_string: &str,
) -> Result<Command,String> {
    let cmd_string = String::from(raw_cmd_string);
    let mut words = cmd_string.split_whitespace();
    
    //Get the Program to Run
    let program =  words.next();
    if program.is_none(){
        return Err(String::from("Empty Program"));
    }
    let mut command = Command::new(program.unwrap());

    while let Some(w) = words.next(){
        match w {
            //Redirections
            "<" | ">" => {
                let filename = words.next();
                
                if filename.is_none(){ 
                    return Err(String::from("Empty File redirection")); 
                }
                let filename = filename.unwrap();

                let file = File::open(filename).expect("Failed Opening file");

                if w == "<" {
                    command.stdin(Stdio::from(file));
                }
                else{
                    command.stdout(Stdio::from(file));
                }
            }
            //Arguments
            _ => {
                command.arg(&w);
            },
        }
    }
    dbg!(&command);
    Ok(command)
}

///
//runcmd
/*
run_command(command, file_descriptors) -> Result<>
runs a single command, creates subprocess, sets file_descriptors

COMMAND ARGS <INPUT >
INPUT: [n]<word
OUTPUT: [n]>[|]word
*/
// pub fn run_command(
//     raw_cmd_string: &str,
//     stdin: Option<Stdio>,
//     stdout: Option<Stdio>,
//     stderr: Option<Stdio>,
// ) -> Result<ChildStdout> {
//     let mut words = String::from(raw_cmd_string);
//     let mut words = words.split_ascii_whitespace();

//     let command = words.next().unwrap();
//     let mut args: Vec<&str> = vec![];

//     let stdin = stdin.unwrap_or(Stdio::inherit());
//     let stdout = stdout.unwrap_or(Stdio::inherit());
//     let stderr = stderr.unwrap_or(Stdio::inherit());

//     //Parse the rest
//     //TODO: Try to get input / output file
//     while let Some(w) = words.next() {
//         match w {
//             _ => args.push(w),
//         }
//     }

//     let mut child = Command::new(command)
//         .args(args)
//         .stdin(stdin)
//         .stdout(stdout)
//         .stderr(stderr)
//         .spawn()?;
//     child.wait();
//     Ok(child.stdout.unwrap())
// }

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn test_simple_cmd(){
        let c = parse_command("ls");
        assert_eq!(c.is_ok(),true);
        let mut c = c.unwrap();
        let output = c.output().expect("Failed LS");
        assert_eq!(output.status.success(),true);
    }

    #[test]
    fn test_simple_cmd_existing_input(){
        let c = parse_command("ls -la < tests/input > tests/output");
        assert_eq!(c.is_ok(),true);
    }
    #[test]
    fn test_simple_cmd_non_existing_input(){
        let c = parse_command("ls -la < tests/inputs > tests/output");
        assert_eq!(c.is_ok(),false);
    }    


    fn test_simple_cmd_create_new_output(){
        let c = parse_command("ls -la < tests/input > tests/output_new");
        assert_eq!(c.is_ok(),true);
    }

    fn test_simple_cmd_overwrite_output(){
        let c = parse_command("ls -la < tests/input > tests/output");
        assert_eq!(c.is_ok(),true);
    }

    fn test_simple_cmd_append_output(){
        let c = parse_command("ls -la < tests/input > tests/output");
        assert_eq!(c.is_ok(),true);
    }    

    fn test_simple_cmd_redir_stderr(){
        let c = parse_command("ls -la < tests/input > tests/output");
        assert_eq!(c.is_ok(),true);
    }    

}