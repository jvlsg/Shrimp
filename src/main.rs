use std::io::{self, Write};

use shrimp::{preprocessor::{expand,read_input,ExpansionError}, Pipeline};

fn main() {
    loop {
        //PROMPT
        print!("$ ");
        io::stdout().flush().unwrap();

        //TODO modularize this in a "InputHandler"
        let mut line_buffer = String::new();

        let mut split_input = read_input();

        // loop {
        //     io::stdin().read_line(&mut line_buffer).unwrap();

        //     match expand(&line_buffer, &mut split_input){
        //         //CURRENT PROBLEM - IF THERE'S AN UNCLOSED TERMINATOR (PAIR NOT FOUND), WE NEED TO READ MORE UNTIL WE FIND A TERMINATOR. AT THE SAME TIME, WHEN WE READ THE NEXT LINE, WE NEED TO STORE THE LAST STATUS, I.E. WE NEED TO KNOW THAT WE'LL BE LOOKING FOR THE PAIR. OR IN THE CASE OF LINES ENDING WITH `\` UNTIL THERE'S A LINE WITHOUT `\`

        //         //WHAT IF WE ENCAPSULATE THIS LOGIC INTO A "INPUT HANDLER" THAT WILL LOOP UNTIL IT GETS A OK(DONE) FROM THE PREPROCESSOR, AND STORES THE LAST RESULTS IN A STACK OF SOME SORT TO HANDLE EXCEPTIONS?

        //         //E.G. input handler reads line, passes to preprocessor which finds only one `'` and returns OK(PairNotFound(')). Input processor reads next line and feeds it to the preprocessor. But ideally it would load it straight into the Singlequote expansion. Should we pass the last result as an argument? Or the "llast results" stack?

        //         //What if we get the mainline from the preprocessor into the input handler main loop? 
        //         Ok(_) => {break},
        //         Err(ExpansionError::PairNotFound(c)) => {
        //             //
        //         }
        //     }
            
        //     // if buffer.ends_with("\\\n") {
        //     //     buffer.pop();
        //     //     buffer.pop();
        //     // } else {
        //     //     break;
        //     // }
        // }

        //TODO get possible errors, e.g. if PairNotFound, continues reading
        // let input = expand(&buffer);

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
