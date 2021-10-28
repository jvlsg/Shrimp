use std::{env, fs, path};

use dirs;

enum ExpansionError {
    PairNotFound,
}

// TODO switch to result
pub fn expand(input: &str) -> Vec<String> {
    let mut split_input: Vec<String> = Vec::with_capacity(input.len()); //Worst case scenario, each char is whitespace separated
    let mut temp_buffer = String::with_capacity(input.len());

    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '$' => {
                expand_env_var(&mut chars, &mut temp_buffer);
            }
            '*' | '?' | '[' => { //Expand until the next non-special character
                 // expand_pathname(&mut chars, &mut temp_buffer);
            }
            '~' => {
                if let Some(home) = dirs::home_dir() {
                    temp_buffer.push_str(home.to_str().unwrap_or_default());
                }
                //TODO else, log?
            }
            '\'' => {
                single_quote_supression(&mut chars, &mut temp_buffer);
            }
            '\"' => {
                double_quote_supression(&mut chars, &mut temp_buffer);
            }
            '\\' => {

            }
            _ if c.is_whitespace() => {
                dbg!(&temp_buffer);
                //We don't want to clone a String with the same capacity, this will allocate only the needed space
                split_input.push(temp_buffer.as_str().to_string());
                temp_buffer.clear();
            }
            _ => {
                temp_buffer.push(c);
            }
        }
    }

    //sanity checking
    if !temp_buffer.is_empty() {
        // expanded_input.push_str(&temp_buffer);
        split_input.push(temp_buffer.as_str().to_string());
    }

    dbg!(&split_input);
    split_input
}

fn expand_env_var(chars: &mut std::iter::Peekable<std::str::Chars>, temp_buffer: &mut String) {
    //Get var name
    let var_name = get_next_word_whitespace_separated(chars);
    dbg!(&var_name);
    if let Ok(value) = env::var(var_name) {
        temp_buffer.push_str(&value);
    }
}

//Todo
fn expand_pathname(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    temp_buffer: &mut String,
) -> std::io::Result<()> {
    //See contents of temp_buffer - if it forms a path
    //Get before * -
    let mut dir_path = if temp_buffer.is_empty() {
        std::env::current_dir()?
    } else {
        path::PathBuf::from(&temp_buffer)
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

    let after_star = get_next_word_whitespace_separated(chars);

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
fn single_quote_supression(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    temp_buffer: &mut String,
) -> Result<(), ExpansionError> {
    let mut found_pair = false;

    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                found_pair = true;
                break;
            }
            _ => {
                //preserve all characters including whitespace
                temp_buffer.push(c);
            }
        }
    }

    //If not found pair, return error
    if !found_pair {
        return Err(ExpansionError::PairNotFound);
    }
    Ok(())
}

/// Supresses all expansions, with the exception of $ and \ expansion
fn double_quote_supression(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    temp_buffer: &mut String,
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
    //             temp_buffer.push(c);
    //         }
    //     }
    // }

    // //If not found pair, return error
    // if !found_pair {
    //     return Err(ExpansionError::PairNotFound);
    // }
    // Ok(())
}


fn get_next_word_whitespace_separated(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut next_word = String::new();

    while let Some(c) = chars.peek() {
        if c.is_whitespace() {
            break;
        }
        next_word.push(chars.next().unwrap());
    }
    next_word
}

mod test {
    use super::*;
    #[test]
    fn simple_env() {
        let key = "KEY";
        env::set_var(key, "VALUE");
        assert_eq!(
            expand("echo $KEY"),
            vec![String::from("echo"), String::from("VALUE")]
        );
    }

    #[test]
    fn simple_pathname() {
        // expand("my/dir/file*.txt");
        // expand("my/dir/*.txt");
        // expand("file*.txt");
        // expand("*.txt");

        // expand("../src/main*s");
        // expand("../src/*.txt");
        //TODO FAILING HERE
        expand("m*.rs");
        // expand("*.rs");
    }
}
