use std::{
    env, fs,
    io::{self, Write},
    path::{Component, Path, PathBuf},
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
    let mut input_raw = String::new();

    let mut split_input = vec![];

    read_line_into_main_prompt(&mut input_raw);

    loop {
        match expand(&input_raw, &mut split_input) {
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
    let mut expanded_input: Vec<String> = Vec::with_capacity(input_raw.len()); //Worst case scenario, each char is whitespace separated
    let mut curr_expansion_buffer = String::with_capacity(input_raw.len());

    let mut new_line_buffer = String::new();
    let mut leftover_buffer = String::new();
    let mut input_iter = input_raw.chars().peekable();
    while let Some(c) = input_iter.next() {
        match c {
            '$' => {
                input_iter = set_owner_get_chars_peekable(
                    expand_env_var(input_iter.by_ref().collect(), &mut curr_expansion_buffer),
                    &mut leftover_buffer,
                );
            }
            '*' => {
                //TODO Check if there's another * to perform recursive?:w
                input_iter = set_owner_get_chars_peekable(
                    expand_pathname_wildcard(
                        input_iter.by_ref().collect(),
                        &mut curr_expansion_buffer,
                    ),
                    &mut leftover_buffer,
                );
            }
            // '?' | '[' => {
            //     //Expand until the next non-special character
            //     expand_pathname(input_iter.by_ref().collect(), &mut curr_expansion_buffer);
            // }
            '~' => {
                if let Some(next_char) = input_iter.peek() {
                    if *next_char == '/' || next_char.is_whitespace() {
                        if let Some(home) = dirs::home_dir() {
                            curr_expansion_buffer.push_str(home.to_str().unwrap_or_default());
                        }
                    }
                }

                //TODO else, log?
            }
            '\'' => {
                input_iter = set_owner_get_chars_peekable(
                    single_quote_supression(
                        input_iter.by_ref().collect(),
                        &mut curr_expansion_buffer,
                    ),
                    &mut leftover_buffer,
                );
            }
            '\"' => {
                input_iter = set_owner_get_chars_peekable(
                    double_quote_supression(
                        input_iter.by_ref().collect(),
                        &mut curr_expansion_buffer,
                    ),
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
                        curr_expansion_buffer.push(input_iter.next().unwrap());
                    }
                    None => (),
                }
            }
            _ if c.is_whitespace() => {
                //We don't want to **clone** a String with the same capacity as curr_expansion_buffer
                // .as_str().to_string() will allocate only the needed space,
                // while ownership of curr_expansion_buffer remains in this function
                expanded_input.push(curr_expansion_buffer.as_str().to_string());
                curr_expansion_buffer.clear();
            }
            _ => {
                curr_expansion_buffer.push(c);
            }
        }
    }

    //sanity checking to avoid adding empty String to split input
    if !curr_expansion_buffer.is_empty() {
        expanded_input.push(curr_expansion_buffer.as_str().to_string());
    }

    dbg!(&expanded_input);
    input_processed.append(&mut expanded_input);
    Ok(())
}

/** Implementation for the functions that handle each expansion.
 The convention is that the function takes ownership of the non-processed input_buffer,
    and a mutable reference to processed curr_expanded_buffer as args.
The function then performs it's expansion, pushing the new characters to curr_expanded_buffer. It then returns all remaining characters.
*/

/// Replaces the first string composed of alphanumeric and `_` with the value of a environment variable of the same name, or blank "" as a default
/// Returns any leftover input
fn expand_env_var(input_buffer: String, curr_expanded_buffer: &mut String) -> String {
    //Get var name
    //Get up until a delimiter... i.e. read alphanumeric and _
    let (var_name, _) = input_buffer
        .split_once(|c| !char::is_alphanumeric(c) && c != '_')
        .unwrap_or((&input_buffer, &""));
    // .unwrap_or_default();

    dbg!(&var_name);
    if let Ok(value) = env::var(var_name) {
        curr_expanded_buffer.push_str(&value);
    }
    input_buffer
        .strip_suffix(var_name)
        .unwrap_or_default()
        .to_owned()
}

fn expand_pathname_wildcard(input_buffer: String, curr_expanded_buffer: &mut String) -> String {
    // base_dir/{prefix}*{suffix}/child_path <======================================

    // (base_dir) We need the most complete path possible (i.e. the nearest dir) up until '*' (remember it can appear in the middle), defaulting to CWD
    // (prefix) We need the remainder between the base_dir and the '*', if any. Beginning of the
    // (suffix) Everything in the file name after '*' , e.g. extensions, etc.
    // (child_path) sub-directories/files that are under the dirs to be expanded

    // We can have multiple, separate '*' - my/dir/*/sub_dir/*
    // But this will ONLY happen if star_suffix is a path (has '/'). If it does, then we could LOOP ("recursive")
    // the same logic.
    // (If suffix != empty and start of path). Make current result the new base_dir. Check if there's star_prefix. Loop

    // let mut dir_path = if curr_expanded_buffer.is_empty() {
    //     match std::env::current_dir() {
    //         Ok(path) => path,
    //         Err(_error) => {
    //             eprintln!("No permission access to current dir, or it does not exist");
    //             return String::new();
    //         }
    //     }
    // } else {
    //     path::PathBuf::from(&curr_expanded_buffer)
    // };
    let full_path = PathBuf::from(&curr_expanded_buffer);
    dbg!(&full_path);
    let mut base_dir_path = PathBuf::new();
    let mut wildcard_prefix = String::new();
    let mut wildcard_suffix = String::new();

    let (path_remainder, input_buffer) = input_buffer
        .split_once(|c: char| c.is_whitespace())
        .unwrap_or((&input_buffer, "")); //if no whitespace is present, the rest of input_buffer has to be processed

    let path_remainder = PathBuf::from(path_remainder);

    let mut path_remainder_iter = path_remainder.components();

    if let Some(Component::Normal(first_component)) = path_remainder_iter.next() {
        wildcard_suffix = first_component.to_str().unwrap_or_default().to_string();
    }

    let child_path: PathBuf = path_remainder_iter.collect();

    // Check if path exists. If not, Check if up until the parent it exists, default to PWD
    if full_path.exists() {
        base_dir_path = full_path
    } else {
        // ls Documents/Books/RPG/*.pdf TODO is failing
        // BUG failed wildcard, from wrong file, is crashing ^
        base_dir_path = full_path.parent().unwrap_or(&Path::new("./")).to_path_buf();

        wildcard_prefix = if full_path.is_file() {
            String::from(
                full_path
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default(),
            )
        } else {
            String::new()
        };
    };

    //TODO treat possible Err on read_dir
    //TODO possibly order results?

    // Inside the "ase_dir, we list the contents in that level.
    // Select everything that starts with prefix AND ends with the suffix

    dbg!(&base_dir_path);
    dbg!(&wildcard_prefix);
    dbg!(&wildcard_suffix);

    let mut entries = fs::read_dir(base_dir_path)
        .unwrap()
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().to_os_string().into_string().ok())
        .filter(|e| e.starts_with(&wildcard_prefix) && e.ends_with(&wildcard_suffix))
        .collect::<Vec<String>>();

    // BUG just appending is making things strange
    //ls Desktop/*/test
    //    "dir_btest",
    //        "dir_atest",
    entries
        .iter_mut()
        .for_each(|e| e.push_str(&child_path.to_str().unwrap_or_default()));

    dbg!(&entries);
    //We have a list of all possibilities. To avoid having to implement recursion
    //We append all possibilities to the input_buffer and send it to return to the main loop.

    //TODO If the entries have whitespaces in them, they must be espaced before appending

    let mut joined_entries = entries.join(" ");
    joined_entries.push_str(input_buffer);
    joined_entries
}

/// Supresses all expansions
/// Gets ownership of a String w/ all input provided from the user so far.
/// Reads all input, including new lines if necessary, until a pair to `'` is found
/// Leftover input *after* the `'`, if any, is returned and should be used to update the iterator in the main loop
fn single_quote_supression(curr_input_buffer: String, curr_expanded_buffer: &mut String) -> String {
    let mut found_pair = false;
    let mut next_input_buffer = String::new();
    let mut curr_input_iter = curr_input_buffer.chars();

    while !found_pair {
        while let Some(c) = curr_input_iter.next() {
            dbg!(&curr_expanded_buffer);
            match c {
                '\'' => {
                    found_pair = true;
                    break;
                }
                _ => {
                    //preserve all characters including whitespace
                    curr_expanded_buffer.push(c);
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
fn double_quote_supression(curr_input_buffer: String, curr_expanded_buffer: &mut String) -> String {
    let mut found_pair = false;
    let mut next_input_buffer = String::new();
    let mut curr_input_iter = curr_input_buffer.chars();
    let mut leftover_buffer = String::new();

    while !found_pair {
        while let Some(c) = curr_input_iter.next() {
            dbg!(&curr_expanded_buffer);
            match c {
                '$' => {
                    curr_input_iter = set_owner_get_chars(
                        expand_env_var(curr_input_iter.by_ref().collect(), curr_expanded_buffer),
                        &mut leftover_buffer,
                    );
                }
                '\"' => {
                    found_pair = true;
                    break;
                }
                _ => {
                    //preserve all characters including whitespace
                    curr_expanded_buffer.push(c);
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
    #[test]
    fn success_expand_env() {
        let key = "SOME_KEY";
        env::set_var(key, "VALUE");

        let mut input_expanded = vec![];

        assert!(expand("echo $SOME_KEY", &mut input_expanded).is_ok());
        assert_eq!(
            input_expanded,
            vec![String::from("echo"), String::from("VALUE")]
        );
    }

    #[test]
    fn fail_expand_env_non_existing_env() {
        let mut input_expanded = vec![];

        assert!(expand("echo $NON_EXISTING", &mut input_expanded).is_ok());
        assert_eq!(input_expanded, vec![String::from("echo")]);
    }

    #[test]
    fn success_single_quote() {
        let mut input_expanded = vec![];
        //user@pc$: bla '~" $HOME\*'
        assert!(expand("bla '~\" $HOME\\*' ", &mut input_expanded).is_ok());

        assert_eq!(
            input_expanded,
            vec![String::from("bla"), "~\" $HOME\\*".to_owned()]
        );
    }
    #[test]
    fn success_double_quote() {
        let key = "SOME_KEY";
        env::set_var(key, "VALUE");

        let mut input_expanded = vec![];

        assert!(expand("bla \"~ $SOME_KEY ./*'\"", &mut input_expanded).is_ok());

        assert_eq!(
            input_expanded,
            vec![String::from("bla"), "~ VALUE ./*'".to_owned()]
        );
    }

    #[test]
    fn success_wildcard_pathname() {
        let mut input_expanded = vec![];

        assert!(expand("*.toml", &mut input_expanded).is_ok());
        assert_eq!(input_expanded, vec!["Cargo.toml".to_owned()]);

        input_expanded.clear();

        assert!(expand("./*.toml", &mut input_expanded).is_ok());
        assert_eq!(input_expanded, vec!["./Cargo.toml".to_owned()]);
        // expand("my/dir/file*.txt");
        // expand("my/dir/*.txt");
        // expand("file*.txt");
        // expand("*.txt");

        // fs::create_dir("tests/dir/sub_dir_1").unwrap();
        // fs::create_dir("tests/dir/sub_dir_2").unwrap();

        // expand("../src/main*s");
        // expand("../src/*.txt");
        //TODO FAILING HERE
        // expand("m*.rs");
        // expand("*.rs");
        // expand("/Desktop/*/test"); // Gives Desktop/dir_a/test and Desktop/dir_b/test
        // expand("/Desktop/*/test/*/test2"); // Gives Desktop/dir_a/test/subidr_a/test2 and Desktop/dir_b/test/subdir_b/test2
    }

    #[test]
    fn success_wildcard_subdir() {
        let test_dir = PathBuf::from("tests/dir");
        let mut file_1 = test_dir.clone();
        file_1.push("file_1.txt");

        fs::create_dir(&test_dir).unwrap();
        fs::File::create(&file_1).unwrap();

        let mut input_expanded = vec![];

        assert!(expand("tests/dir/*.txt", &mut input_expanded).is_ok());
        assert_eq!(input_expanded, vec!["tests/dir/file_1.txt".to_owned()]);

        input_expanded.clear();

        assert!(expand("./tests/dir/*.txt", &mut input_expanded).is_ok());
        assert_ne!(input_expanded, vec!["./tests/dir/file_1.txt".to_owned()]);

        fs::remove_file(&file_1).unwrap();
        fs::remove_dir(&test_dir).unwrap();
    }
}
