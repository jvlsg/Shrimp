use shrimp::{
    input_handler::{InputHandler, InputHandlingError},
    Config, Pipeline,
};

fn main() {
    let mut input_handler = InputHandler::new(Config::new());

    loop {
        match input_handler.read_user_input() {
            // 3. Implement sublists, pipelines separated with && and ||
            // 4. Implement a List
            Ok(split_input) => {
                if let Ok(p) = Pipeline::new(split_input) {
                    match p.run() {
                        Ok(_) => {}
                        Err(msg) => {
                            eprintln!("{}", msg);
                        }
                    }
                }
            }
            Err(e) => match e {
                InputHandlingError::Expansion(f) => eprintln!("{}", f),
                InputHandlingError::ReadLine(_) => {
                    break;
                }
            },
        }
    }
}
