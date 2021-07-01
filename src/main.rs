use std::{
    io::{self, Write},
    process::ExitStatus,
};

use shrimp::{Pipeline, Step};

fn main() {
    loop {
        //PROMPT
        print!("> ");
        io::stdout().flush().unwrap();
        //READ A RAW LINE
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        let input = buffer.trim();

        //Lets implement bottom-up
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
                let mut p = Pipeline::new(input).unwrap();
                p.run().unwrap();
                // dbg!(input);
                // match Step::parse_command(input) {
                //     //Valid Syntax, File paths, etc.
                //     Ok(mut c) => {
                //         if let Ok(mut child) = c.spawn() {
                //             child.wait();
                //         }
                //         //TODO Improve error handling
                //     }
                //     Err(msg) => {
                //         eprintln!("{}", msg);
                //     }
                // }
            }
        }
    }
}
