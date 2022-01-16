use std::io::{self, Write};

use shrimp::{
    preprocessor::{expand, read_user_input, ExpansionError},
    Pipeline,
};

fn main() {
    loop {

        let split_input = read_user_input();

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
    }
}
