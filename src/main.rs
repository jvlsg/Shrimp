use std::io::{self, Write};

use shrimp::Pipeline;

fn main() {
    loop {
        //PROMPT
        print!("> ");
        io::stdout().flush().unwrap();
        //READ A RAW LINE
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        let input = buffer.trim();

        //3. Implement sublists, pipelines separated with && and ||
        //4. Implement a List

        match input {
            "exit" | "quit" => break,

            _ => {
                if let Ok(p) = Pipeline::new(input) {
                    match p.run() {
                        Ok(_) => {}
                        Err(msg) => {
                            eprintln!("{}", msg);
                        }
                    }
                }
            }
        }
    }
}
