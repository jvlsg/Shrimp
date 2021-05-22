use std::{io, process::ExitStatus};

// use shrimp::{run_command, Pipeline};

fn main() {
    loop {
        //READ A RAW LINE
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        let input = buffer.trim();

        //Lets implement bottom-up
        //1. Read input and create a child process
        //2. Implement pipelines w/ vec of commands and management of redirections
        //3. Implement sublists, pipelines separated with && and ||
        //4. Implement a List

        //Check for Builtins
        match input {
            "exit" | "quit" => break,
            // _ => match run_command(input, None, None, None) {
            //     Ok(_) => continue,
            //     Err(e) => {
            //         eprintln!("{}", e);
            //         continue;
            //     }
            // },
            _ => {
                // let p = Pipeline::new(input);
                // p.run();
                dbg!(input);
            }
        }
    }
}
