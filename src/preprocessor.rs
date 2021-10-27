use std::{
    env,
    fs,
    path,
    io::Result
};

use dirs;

// TODO switch to result
pub fn expand(input: &str) -> String {
    let mut expanded_input = String::with_capacity(input.len());
    let mut temp_buffer = String::with_capacity(input.len());

    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '$' => {
                expand_env_var(&mut chars, &mut temp_buffer);
            }
            '*' => {//Expand until the next non-special character
                // expand_pathname(&mut chars, &mut temp_buffer);
                
            }
            '~' => {
                if let Some(home) = dirs::home_dir() {
                    temp_buffer.push_str(home.to_str().unwrap_or_default());
                }
                //TODO else, log?
                
            }            
            _ if c.is_whitespace() => {
                dbg!(&temp_buffer);
                expanded_input.push_str(&temp_buffer);
                expanded_input.push(c);
                temp_buffer.clear();
            }
            _ => {
                temp_buffer.push(c);
            }
        }
    }

    //sanity checking
    if !temp_buffer.is_empty() {
        expanded_input.push_str(&temp_buffer);
    }

    dbg!(&expanded_input);
    expanded_input
}

fn expand_env_var(chars: &mut std::iter::Peekable<std::str::Chars>, temp_buffer: &mut String) {
    //Get var name
    let var_name = get_next_word_whitespace(chars);
    dbg!(&var_name);
    if let Ok(value) = env::var(var_name) {
        temp_buffer.push_str(&value);
    }
}

//Todo
fn expand_pathname(chars: &mut std::iter::Peekable<std::str::Chars>, temp_buffer: &mut String) -> std::io::Result<()> {
    //See contents of temp_buffer - if it forms a path
    //Get before * -
    let mut dir_path = if temp_buffer.is_empty() {std::env::current_dir()?} else {path::PathBuf::from(&temp_buffer)};
    //Beginning of filename, if exists
    let mut before_star = None;

    // What if after * is another *? Recursive?
    if let Some(file_name) = dir_path.file_name() {

        match file_name.to_str(){
            Some(s)=>{ before_star = Some(String::from(s)) }
            None => {before_star = None}
        }
    
    }
    
    dir_path.pop();


    let after_star = get_next_word_whitespace(chars);
    
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


fn get_next_word_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
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
        assert_eq!(expand("echo $KEY\n"), String::from("echo VALUE\n"));
    }

    #[test]
    fn simple_pathname(){
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
