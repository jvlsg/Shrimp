use std::{
    env, fs,
    io::{self, Write},
    path,
};

use dirs;

//POSSIBLY - CHANGE EXPANSION ERROR FOR ACTUAL ERRORS. HAVE A ENUM FOR THE OK STATUSES THAT THE INPUT HANDLER WILL NEED TO HANDLE
pub enum ExpansionError {
    EnvVarError,
}

fn read_line_into_main_prompt(buf: &mut String) {
    //PROMPT
    print!("$ ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(buf).unwrap();
}

fn read_line_into_secondary_prompt(buf: &mut String) {
    //PROMPT
    print!("> ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(buf).unwrap();
}

pub fn read_user_input() -> Vec<String> {
    //TODO modularize this in a "InputHandler"
    let mut input_raw = String::new();

    let mut split_input = vec![];

    read_line_into_main_prompt(&mut input_raw);

    loop {
        match expand(&input_raw, &mut split_input) {
            //CURRENT PROBLEM - IF THERE'S AN UNCLOSED TERMINATOR (PAIR NOT FOUND), WE NEED TO READ MORE UNTIL WE FIND A TERMINATOR. AT THE SAME TIME, WHEN WE READ THE NEXT LINE, WE NEED TO STORE THE LAST STATUS, I.E. WE NEED TO KNOW THAT WE'LL BE LOOKING FOR THE PAIR. OR IN THE CASE OF LINES ENDING WITH `\` UNTIL THERE'S A LINE WITHOUT `\`

            //WHAT IF WE ENCAPSULATE THIS LOGIC INTO A "INPUT HANDLER" THAT WILL LOOP UNTIL IT GETS A OK(DONE) FROM THE PREPROCESSOR, AND STORES THE LAST RESULTS IN A STACK OF SOME SORT TO HANDLE EXCEPTIONS?

            //E.G. input handler reads line, passes to preprocessor which finds only one `'` and returns OK(PairNotFound(')). Input processor reads next line and feeds it to the preprocessor. But ideally it would load it straight into the Singlequote expansion. Should we pass the last result as an argument? Or the "llast results" stack?

            //What if we get the mainline from the preprocessor into the input handler main loop?
            Ok(_) => break,
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
fn expand(input_raw: &str, input_processed: &mut Vec<String>) -> Result<(), ExpansionError> {
    let mut split_input: Vec<String> = Vec::with_capacity(input_raw.len()); //Worst case scenario, each char is whitespace separated
    let mut expanded_buffer = String::with_capacity(input_raw.len());

    let mut new_line_buffer = String::new();
    let mut leftover_buffer = String::new();
    let mut input_iter = input_raw.chars().peekable();
    while let Some(c) = input_iter.next() {
        match c {
            '$' => {
                input_iter = set_owner_get_chars_peekable(
                    expand_env_var(input_iter.by_ref().collect(), &mut expanded_buffer),
                    &mut leftover_buffer,
                );
            }
            '*' | '?' | '[' => { //Expand until the next non-special character
                 // expand_pathname(&mut chars, &mut expanded_buffer);
            }
            '~' => {
                if let Some(next_char) = input_iter.peek() {
                    if *next_char == '/' || next_char.is_whitespace() {
                        if let Some(home) = dirs::home_dir() {
                            expanded_buffer.push_str(home.to_str().unwrap_or_default());
                        }
                    }
                }

                //TODO else, log?
            }
            '\'' => {
                input_iter = set_owner_get_chars_peekable(
                    single_quote_supression(input_iter.by_ref().collect(), &mut expanded_buffer),
                    &mut leftover_buffer,
                );
            }
            '\"' => {
                input_iter = set_owner_get_chars_peekable(
                    double_quote_supression(input_iter.by_ref().collect(), &mut expanded_buffer),
                    &mut leftover_buffer,
                );
            }
            '\\' => {
                //Supresses the next character. Exception taken for '\n', in that case read the next line and process it.
                match input_iter.peek() {
                    Some('\n') => {
                        read_line_into_secondary_prompt(&mut new_line_buffer);
                        input_iter = new_line_buffer.chars().peekable();
                    }
                    Some(_) => {
                        expanded_buffer.push(input_iter.next().unwrap());
                    }
                    None => (),
                }
            }
            _ if c.is_whitespace() => {
                //We don't want to **clone** a String with the same capacity as expanded_buffer
                // .as_str().to_string() will allocate only the needed space,
                // while ownership of expanded_buffer remains in this function
                split_input.push(expanded_buffer.as_str().to_string());
                expanded_buffer.clear();
            }
            _ => {
                expanded_buffer.push(c);
            }
        }
    }

    //sanity checking to avoid adding empty String to split input
    if !expanded_buffer.is_empty() {
        split_input.push(expanded_buffer.as_str().to_string());
    }

    dbg!(&split_input);
    input_processed.append(&mut split_input);
    Ok(())
}

/// Replaces the first string composed of alphanumeric and `_` with the value of a environment variable of the same name, or blank "" as a default
/// Returns any leftover input
fn expand_env_var(input_buffer: String, expanded_buffer: &mut String) -> String {
    //Get var name
    //Get up until a delimiter... i.e. read alphanumeric and _
    let (var_name, _) = input_buffer
        .split_once(|c| !char::is_alphanumeric(c) && c != '_')
        .unwrap_or_default();

    dbg!(&var_name);
    if let Ok(value) = env::var(var_name) {
        expanded_buffer.push_str(&value);
    }
    input_buffer
        .strip_prefix(var_name)
        .unwrap_or_default()
        .to_owned()
}

//Todo
fn expand_pathname(
    input_iter: &mut std::iter::Peekable<std::str::Chars>,
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

    let after_star = get_next_word_whitespace_separated(input_iter);

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
/// Gets ownership of a String w/ all input provided from the user so far.
/// Reads all input, including new lines if necessary, until a pair to `'` is found
/// Leftover input *after* the `'`, if any, is returned and should be used to update the iterator in the main loop
fn single_quote_supression(curr_input_buffer: String, expanded_buffer: &mut String) -> String {
    let mut found_pair = false;
    let mut next_input_buffer = String::new();
    let mut curr_input_iter = curr_input_buffer.chars();

    while !found_pair {
        while let Some(c) = curr_input_iter.next() {
            dbg!(&expanded_buffer);
            match c {
                '\'' => {
                    found_pair = true;
                    break;
                }
                _ => {
                    //preserve all characters including whitespace
                    expanded_buffer.push(c);
                }
            }
        }
        if !found_pair {
            dbg!("Reading more input");
            next_input_buffer.clear();
            read_line_into_secondary_prompt(&mut next_input_buffer);
            curr_input_iter = next_input_buffer.chars();
        }
    }
    curr_input_iter.collect()
}

/// Supresses all expansions, with the exception of $ and \ expansion
/// Gets ownership of a String w/ all input provided from the user so far.
/// Reads all input, including new lines if necessary, until a pair to `"` is found
/// Leftover input *after* the `"`, if any, is returned and should be used to update the iterator in the main loop
fn double_quote_supression(curr_input_buffer: String, expanded_buffer: &mut String) -> String {
    let mut found_pair = false;
    let mut next_input_buffer = String::new();
    let mut curr_input_iter = curr_input_buffer.chars();
    let mut leftover_buffer = String::new();

    while !found_pair {
        while let Some(c) = curr_input_iter.next() {
            dbg!(&expanded_buffer);
            match c {
                '$' => {
                    curr_input_iter = set_owner_get_chars(
                        expand_env_var(curr_input_iter.by_ref().collect(), expanded_buffer),
                        &mut leftover_buffer,
                    );
                }
                '\"' => {
                    found_pair = true;
                    break;
                }
                _ => {
                    //preserve all characters including whitespace
                    expanded_buffer.push(c);
                }
            }
        }
        if !found_pair {
            dbg!("Reading more input");
            next_input_buffer.clear();
            read_line_into_secondary_prompt(&mut next_input_buffer);
            curr_input_iter = next_input_buffer.chars();
        }
    }
    curr_input_iter.collect()
}

fn get_next_word_whitespace_separated(
    input_iter: &mut std::iter::Peekable<std::str::Chars>,
) -> String {
    let mut next_word = String::new();

    while let Some(c) = input_iter.peek() {
        if c.is_whitespace() {
            break;
        }
        next_word.push(input_iter.next().unwrap());
    }
    next_word
}

/// Stores the a value (usually from a function) into a **longer living** owner variable. Returns the chars iterator of the buffer
fn set_owner_get_chars(value: String, owner: &mut String) -> std::str::Chars {
    *owner = value;
    owner.chars()
}

fn set_owner_get_chars_peekable(
    value: String,
    owner: &mut String,
) -> std::iter::Peekable<std::str::Chars> {
    set_owner_get_chars(value, owner).peekable()
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
