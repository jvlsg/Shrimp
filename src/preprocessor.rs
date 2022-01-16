use std::{env, fs, path, io};

use dirs;

//POSSIBLY - CHANGE EXPANSION ERROR FOR ACTUAL ERRORS. HAVE A ENUM FOR THE OK STATUSES THAT THE INPUT HANDLER WILL NEED TO HANDLE
pub enum ExpansionError {
    PairNotFound(char),
    EnvVarError
}

pub fn read_input() -> Vec<String> {

    //TODO modularize this in a "InputHandler"
    let mut input_raw = String::new();

    let mut split_input = vec![];

    io::stdin().read_line(&mut input_raw).unwrap();

    loop {
    match expand(&input_raw, &mut split_input){
        //CURRENT PROBLEM - IF THERE'S AN UNCLOSED TERMINATOR (PAIR NOT FOUND), WE NEED TO READ MORE UNTIL WE FIND A TERMINATOR. AT THE SAME TIME, WHEN WE READ THE NEXT LINE, WE NEED TO STORE THE LAST STATUS, I.E. WE NEED TO KNOW THAT WE'LL BE LOOKING FOR THE PAIR. OR IN THE CASE OF LINES ENDING WITH `\` UNTIL THERE'S A LINE WITHOUT `\`

        //WHAT IF WE ENCAPSULATE THIS LOGIC INTO A "INPUT HANDLER" THAT WILL LOOP UNTIL IT GETS A OK(DONE) FROM THE PREPROCESSOR, AND STORES THE LAST RESULTS IN A STACK OF SOME SORT TO HANDLE EXCEPTIONS?

        //E.G. input handler reads line, passes to preprocessor which finds only one `'` and returns OK(PairNotFound(')). Input processor reads next line and feeds it to the preprocessor. But ideally it would load it straight into the Singlequote expansion. Should we pass the last result as an argument? Or the "llast results" stack?

        //What if we get the mainline from the preprocessor into the input handler main loop? 
        Ok(_) => {break},
        // Err(ExpansionError::PairNotFound(c)) => {
        //     //
        // }
        _ => {
            dbg!("Erro");
            ()
        }
    }
    }
    split_input
}

///Handles expansions / metacharacters the user can input on a line.
pub fn expand(input_raw: &str, input_processed: &mut Vec<String>) -> Result<(),ExpansionError> {
    
    let mut split_input: Vec<String> = Vec::with_capacity(input_raw.len()); //Worst case scenario, each char is whitespace separated
    let mut expanded_buffer = String::with_capacity(input_raw.len());

    let mut leftover_string = String::new();
    let mut input_buffer = input_raw.chars().peekable();

    while let Some(c) = input_buffer.next() {
        match c {
            '$' => {
                expand_env_var(&mut input_buffer, &mut expanded_buffer);
            }
            '*' | '?' | '[' => { //Expand until the next non-special character
                 // expand_pathname(&mut chars, &mut expanded_buffer);
            }
            '~' => {
                if let Some(home) = dirs::home_dir() {
                    expanded_buffer.push_str(home.to_str().unwrap_or_default());
                }
                //TODO else, log?
            }
            // '\'' => {
            //     let leftover_stringo = single_quote_supression(&mut input_buffer, &mut expanded_buffer);
            //     if leftover_stringo.is_some(){
            //         leftover_string = leftover_stringo.unwrap();
            //         input_buffer = leftover_string.chars().peekable()
            //     }
            // }
            '\"' => {
                double_quote_supression(&mut input_buffer, &mut expanded_buffer);
            }
            '\\' => {

            }
            _ if c.is_whitespace() => {
                dbg!(&expanded_buffer);
                //We don't want to clone a String with the same capacity, this will allocate only the needed space
                split_input.push(expanded_buffer.as_str().to_string());
                expanded_buffer.clear();
            }
            _ => {
                expanded_buffer.push(c);
            }
        }
    }

    //sanity checking
    if !expanded_buffer.is_empty() {
        // expanded_input.push_str(&expanded_buffer);
        split_input.push(expanded_buffer.as_str().to_string());
    }

    dbg!(&split_input);
    input_processed.append(&mut split_input);
    Ok(())
}

fn expand_env_var(input_buffer: &mut std::iter::Peekable<std::str::Chars>, expanded_buffer: &mut String) {
    //Get var name
    let var_name = get_next_word_whitespace_separated(input_buffer);
    dbg!(&var_name);
    if let Ok(value) = env::var(var_name) {
        expanded_buffer.push_str(&value);
    }
}

//Todo
fn expand_pathname(
    input_buffer: &mut std::iter::Peekable<std::str::Chars>,
    expanded_buffer: &mut String,
) -> std::io::Result<()> {
    //See contents of expanded_buffer - if it forms a path
    //Get before * -
    let mut dir_path = if expanded_buffer.is_empty() {
        std::env::current_dir()?
    } else {
        path::PathBuf::from(&expanded_buffer)
    };
    //Beginning of filename, if exists
    let mut before_star = None;

    // What if after * is another *? Recursive?
    if let Some(file_name) = dir_path.file_name() {
        match file_name.to_str() {
            Some(s) => before_star = Some(String::from(s)),
            None => before_star = None,
        }
    }

    dir_path.pop();

    let after_star = get_next_word_whitespace_separated(input_buffer);

    dbg!(&dir_path);
    dbg!(&before_star);
    dbg!(&after_star);

    // fs::read_dir(dir_path)?
    //     .filter_map(|entry| entry.ok())
    //     .filter_map(|entry| entry.file_name().into_string().ok())
    //     .filter(|entry|
    //         if before_star.is_some(){
    //             entry.starts_with(before_star)
    //         }else{

    //         }
    //     );

    Ok(())

    //if before_star not empty ->.starts_with(before_star)
    //if after_star not empty .ends_with(after_star)

    //Loop over all
}

/// Supresses all expansions
fn single_quote_supression(){
//     input_buffer: &mut std::iter::Peekable<std::str::Chars>,
//     expanded_buffer: &mut String,
// ) -> Option<String> { //It might return a next line
    
//     let mut found_pair = false;
//     let mut new_input_buffer = String::new();

//     let mut curr_input = input_buffer;

//     //Reads all input, including new lines if necessary, until a pair to `'` is found
//     while !found_pair {
//         while let Some(c) = curr_input.next() {
//             match c {
//                 '\'' => {
//                     found_pair = true;
//                     break;
//                 }
//                 _ => {
//                     //preserve all characters including whitespace
//                     expanded_buffer.push(c);
//                 }
//             }
//         }
//         //If not found pair, return error
//         if !found_pair {
//             io::stdin().read_line(&mut new_input_buffer).unwrap();
//             curr_input = new
//         }

//     }

//     let leftover_string: String = curr_input.collect();
//     if !leftover_string.is_empty() {Some(leftover_string)} else {None}
unimplemented!();
}

/// Supresses all expansions, with the exception of $ and \ expansion
fn double_quote_supression(
    input_buffer: &mut std::iter::Peekable<std::str::Chars>,
    expanded_buffer: &mut String,
) -> Result<(), ExpansionError> {
    unimplemented!();
    // let mut found_pair = false;

    // while let Some(c) = chars.next() {
    //     match c {
    //         '\'' => {
    //             found_pair = true;
    //             break;
    //         }
    //         _ => {
    //             //preserve all characters including whitespace
    //             expanded_buffer.push(c);
    //         }
    //     }
    // }

    // //If not found pair, return error
    // if !found_pair {
    //     return Err(ExpansionError::PairNotFound);
    // }
    // Ok(())
}


fn get_next_word_whitespace_separated(input_buffer: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut next_word = String::new();

    while let Some(c) = input_buffer.peek() {
        if c.is_whitespace() {
            break;
        }
        next_word.push(input_buffer.next().unwrap());
    }
    next_word
}

mod test {
    use super::*;
//     #[test]
//     fn simple_env() {
//         let key = "KEY";
//         env::set_var(key, "VALUE");
//         assert_eq!(
//             expand("echo $KEY"),
//             vec![String::from("echo"), String::from("VALUE")]
//         );
//     }

//     #[test]
//     fn simple_pathname() {
//         // expand("my/dir/file*.txt");
//         // expand("my/dir/*.txt");
//         // expand("file*.txt");
//         // expand("*.txt");

//         // expand("../src/main*s");
//         // expand("../src/*.txt");
//         //TODO FAILING HERE
//         expand("m*.rs");
//         // expand("*.rs");
//     }
}
