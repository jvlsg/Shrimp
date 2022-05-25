use std::io::{self, Write};

use shrimp::{input_handler, Pipeline};

fn main() {
    loop {
        if let Ok(split_input) = input_handler::read_user_input() {
            //3. Implement sublists, pipelines separated with && and ||
            //4. Implement a List

            if let Ok(p) = Pipeline::new(split_input) {
                match p.run() {
                    Ok(_) => {}
                    Err(msg) => {
                        eprintln!("{}", msg);
                    }
                }
            }
        } else {
            break;
        }
    }
}
