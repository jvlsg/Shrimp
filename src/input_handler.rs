use std::{
    env, error, fmt,
    fmt::Display,
    fs,
    io::{self, Write},
    path::{Component, Path, PathBuf},
};

use dirs;

#[derive(Debug)]
pub enum ExpansionError {
    EnvVar(String),
    WildcardMatch(String),
}

impl From<io::Error> for ExpansionError {
    fn from(error: io::Error) -> Self {
        ExpansionError::WildcardMatch(error.to_string())
    }
}

impl From<env::VarError> for ExpansionError {
    fn from(error: env::VarError) -> Self {
        ExpansionError::EnvVar(error.to_string())
    }
}

impl error::Error for ExpansionError {}

impl Display for ExpansionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            ExpansionError::EnvVar(s) => write!(f, "Env Var error - {}", s),
            ExpansionError::WildcardMatch(s) => write!(f, "Wildcard error - {}", s),
        }
    }
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
            Err(error) => {
                eprintln!("{}", error);
                return vec![];
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
                    expand_env_var(input_iter.by_ref().collect(), &mut curr_expansion_buffer)?,
                    &mut leftover_buffer,
                );
            }
            '*' => {
                input_iter = set_owner_get_chars_peekable(
                    expand_pathname_wildcard(
                        input_iter.by_ref().collect(),
                        &mut curr_expansion_buffer,
                    )?,
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
                    )?,
                    &mut leftover_buffer,
                );
            }
            '\\' => {
                //Supresses the next character. Exception taken for '\n', in that case read the next line and process it.
                // TODO we should expand the line read in the secondary prompt
                match input_iter.peek() {
                    Some('\n') => {
                        new_line_buffer.clear();
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
fn expand_env_var(
    input_buffer: String,
    curr_expanded_buffer: &mut String,
) -> Result<String, env::VarError> {
    //Get var name
    //Get up until a delimiter... i.e. read alphanumeric and _
    let (var_name, _) = input_buffer
        .split_once(|c| !char::is_alphanumeric(c) && c != '_')
        .unwrap_or((&input_buffer, &""));
    // .unwrap_or_default();

    dbg!(&var_name);
    let value = env::var(var_name)?;

    curr_expanded_buffer.push_str(&value);

    Ok(input_buffer
        .strip_suffix(var_name)
        .unwrap_or_default()
        .to_owned())
}

fn expand_pathname_wildcard(
    input_buffer: String,
    curr_expanded_buffer: &mut String,
) -> Result<String, ExpansionError> {
    // base_dir/{prefix}*[{intermediate}*...]{suffix}[/{child_path}]

    // (base_dir) We need the most complete path possible (i.e. the nearest dir) up until '*' (remember it can appear in the middle), defaulting to CWD
    // (prefix) The remainder between base_dir and the first '*', if any.
    // (intermediates) Should there be several '*' in the level, it's the text between two of them
    // (suffix) Everything in the file name after '*' , e.g. extensions, etc.

    //There can be multiple '*' in the same level of the fs hierarchy we're checking.
    // This function will handle this

    // We can have multiple, separate '*' - my/dir/*/sub_dir/* -
    // This function will only handle wildcards ONE level of the fs hierarchy.
    // It will enumerate existing matches and re-add them to the input_buffer for the main loop to re-process

    // Auxiliar function used to select which entries are a match

    fn is_wildcard_match(
        entry: &str,
        prefix: &str,
        intermediates: &Vec<String>,
        suffix: &str,
    ) -> bool {
        return entry.starts_with(&prefix)
            && entry.ends_with(&suffix)
            && intermediates.iter().fold(true, |acc, intermediary| {
                acc && entry.contains(intermediary)
            });
    }

    #[derive(Debug)]
    struct WildcardComponents {
        base_dir_path: PathBuf,
        prefix: Option<String>,
        wildcard_intermediates: Vec<String>,
        wildcard_suffix: Option<String>,
        child_path: PathBuf,
    }
    fn get_wildcard_components(
        curr_expanded_buffer: &str,
        input_buffer: String,
    ) -> Result<(WildcardComponents, String), ExpansionError> {
        let base_dir_and_prefix = PathBuf::from(&curr_expanded_buffer);

        let mut base_dir_path = PathBuf::new();
        let mut prefix: Option<String> = None;
        let mut wildcard_intermediates: Vec<String> = vec![];
        let mut wildcard_suffix: Option<String> = None;

        dbg!(&base_dir_and_prefix);
        // Check if path exists. If not, Check if up until the parent it exists, default to PWD
        if base_dir_and_prefix.exists() {
            base_dir_path = base_dir_and_prefix
        }
        // Non-existing base_dir
        // We have to check the String because PathBuf normalizes trailing slashes
        else if curr_expanded_buffer.ends_with(std::path::MAIN_SEPARATOR) {
            return Err(ExpansionError::WildcardMatch(format!(
                "Non Existing directory {}",
                base_dir_and_prefix.as_os_str().to_str().unwrap_or_default()
            )));
        } else {
            base_dir_path = if base_dir_and_prefix
                .parent()
                .unwrap_or(&Path::new(""))
                .is_dir()
            {
                base_dir_and_prefix.parent().unwrap().to_path_buf()
            } else {
                PathBuf::from("./")
                // std::env::current_dir()?
            };

            prefix = base_dir_and_prefix
                .file_name()
                .unwrap_or_default()
                .to_str()
                .map(String::from)
        };

        let (remaining_path, input_buffer) = input_buffer
            .split_once(|c: char| c.is_whitespace())
            .unwrap_or((&input_buffer, "")); //if no whitespace is present, the rest of input_buffer has to be processed

        // Includes intermediates , other '*' , the suffix and child path
        let remaining_path = PathBuf::from(remaining_path);
        let mut remaining_path_iter = remaining_path.components();

        // Get Intermediates
        if let Some(Component::Normal(first_component)) = remaining_path_iter.next() {
            let mut intermediates_and_suffix = first_component
                .to_str()
                .unwrap_or_default()
                .split("*")
                .collect::<Vec<&str>>();

            wildcard_suffix = intermediates_and_suffix.pop().map(String::from);

            wildcard_intermediates = intermediates_and_suffix
                .into_iter()
                .map(|e| e.to_owned())
                .collect();
        }

        let child_path: PathBuf = remaining_path_iter.collect();

        Ok((
            WildcardComponents {
                base_dir_path,
                prefix,
                wildcard_intermediates,
                wildcard_suffix,
                child_path,
            },
            input_buffer.to_owned(),
        ))
    }

    dbg!(&curr_expanded_buffer);
    dbg!(&input_buffer);

    let (wildcard_components, input_buffer) =
        get_wildcard_components(&curr_expanded_buffer, input_buffer)?;

    dbg!(&wildcard_components);
    let mut entries = fs::read_dir(&wildcard_components.base_dir_path)?
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|e| {
            is_wildcard_match(
                &e.file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default(),
                wildcard_components
                    .prefix
                    .as_ref()
                    .unwrap_or(&String::new()),
                &wildcard_components.wildcard_intermediates,
                wildcard_components
                    .wildcard_suffix
                    .as_ref()
                    .unwrap_or(&String::new()),
            )
        })
        .collect::<Vec<PathBuf>>();

    //BUG if nothing is found, we should return an error
    // Not exactly a bug, but we're failing silently
    //Possibly - instead of adding to input_buffer, we iterate until we run out ?
    // if entries.is_empty() {
    //     curr_expanded_buffer.clear();
    //     return Err(ExpansionError::WildcardMatch("No match found".to_owned()));
    // }

    //If has child path, ignore all files
    if wildcard_components.child_path.capacity() != 0 {
        entries = entries
            .into_iter()
            .filter(|e| e.is_dir())
            .collect::<Vec<_>>();

        entries
            .iter_mut()
            .for_each(|e| e.push(&wildcard_components.child_path));
    }

    let entries = entries
        .iter()
        .filter_map(|e| e.to_str())
        .map(|e| {
            let mut e_owned = String::from(e);
            //remove "./" from the beginning if user didn't explicitly added it
            if e.starts_with("./") && !curr_expanded_buffer.starts_with("./") {
                e_owned = e.trim_start_matches("./").to_owned();
            }
            //If the entries have whitespaces in them, they must be quoted before appending
            if e.contains(|c: char| c.is_whitespace()) {
                e_owned = format!("'{}'", e)
            }
            e_owned
        })
        .collect::<Vec<String>>();

    //Clear curr_expanded buffer so Main loop has a fresh start
    curr_expanded_buffer.clear();

    let joined_entries = if entries.is_empty() {
        input_buffer.to_string()
    } else {
        let joined_entries = entries.join(" ");

        if !input_buffer.is_empty() {
            format!("{} {}", joined_entries, input_buffer)
        } else {
            joined_entries
        }
    };

    dbg!(&joined_entries);
    Ok(joined_entries)
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
fn double_quote_supression(
    curr_input_buffer: String,
    curr_expanded_buffer: &mut String,
) -> Result<String, env::VarError> {
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
                        expand_env_var(curr_input_iter.by_ref().collect(), curr_expanded_buffer)?,
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
    Ok(curr_input_iter.collect())
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

        assert!(expand("Cargo.*", &mut input_expanded).is_ok());
        assert_eq!(
            input_expanded,
            vec!["Cargo.lock".to_owned(), "Cargo.toml".to_owned()]
        );

        input_expanded.clear();

        assert!(expand("./*.toml", &mut input_expanded).is_ok());
        assert_eq!(input_expanded, vec!["./Cargo.toml".to_owned()]);

        input_expanded.clear();

        assert!(expand("./C*r*.toml", &mut input_expanded).is_ok());
        assert_eq!(input_expanded, vec!["./Cargo.toml".to_owned()]);
        //----
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

        let mut input_expanded_1 = vec![];
        let mut input_expanded_2 = vec![];

        let result_1 = expand("tests/dir/*.txt", &mut input_expanded_1);
        let result_2 = expand("./*/d*r/*.txt", &mut input_expanded_2);
        fs::remove_file(&file_1).unwrap();
        fs::remove_dir(&test_dir).unwrap();

        assert!(result_1.is_ok());
        assert_eq!(input_expanded_1, vec!["tests/dir/file_1.txt".to_owned()]);

        assert!(result_2.is_ok());
        assert_eq!(input_expanded_2, vec!["./tests/dir/file_1.txt".to_owned()]);

        let mut input_expanded = vec![];
        assert!(expand("../Shrimp/src/main*s", &mut input_expanded).is_ok());
        assert_eq!(input_expanded, vec!["../Shrimp/src/main.rs".to_owned()]);
    }

    #[test]
    fn fail_wildcard_non_existing_base_dir() {
        let mut input_expanded_1 = vec![];

        let result_1 = expand(
            "../a_folder_that_does_not_exist/*.txt",
            &mut input_expanded_1,
        );
        assert!(result_1.is_err());
    }
}
