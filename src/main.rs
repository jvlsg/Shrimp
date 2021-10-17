use std::io::{self, Write};

use shrimp::{preprocessor::expand, Pipeline};

fn main() {
    loop {
        //PROMPT
        print!("> ");
        io::stdout().flush().unwrap();

        //TODO modularize this in a "InputHandler"
        let mut buffer = String::new();

        loop {
            io::stdin().read_line(&mut buffer).unwrap();
            if buffer.ends_with("\\\n") {
                buffer.pop();
                buffer.pop();
            } else {
                break;
            }
        }

        let input = expand(&buffer);

        //3. Implement sublists, pipelines separated with && and ||
        //4. Implement a List

        if let Ok(p) = Pipeline::new(&input) {
            match p.run() {
                Ok(_) => {}
                Err(msg) => {
                    eprintln!("{}", msg);
                }
            }
        }
    }
}
