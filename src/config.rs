use dirs;
use std::{fs, path::PathBuf};

pub struct Config {
    line_editor_config: rustyline::Config,
    config_dir: PathBuf,
    history_file: PathBuf,
    //TODO prompt, etc.
}

impl Config {
    pub fn new() -> Config {
        // TODO attempt to read
        Config::from_default()
    }

    pub fn from_default() -> Config {
        let line_editor_config = rustyline::config::Builder::new()
            .auto_add_history(true)
            .indent_size(4)
            .completion_type(rustyline::CompletionType::List)
            .max_history_size(2048)
            .build(); // config_path.push(["shrimp", "history"].iter().collect::<PathBuf>());

        let config_dir = match dirs::config_dir() {
            Some(mut config_path) => {
                config_path.push("shrimp");

                if !config_path.exists() {
                    fs::create_dir(&config_path);
                }
                config_path
            }
            None => PathBuf::new(),
        };

        let mut history_file = config_dir.clone();
        history_file.push("shrimp_history");
        if !history_file.exists() {
            fs::write(&history_file, "");
        }

        // dbg!(self.line_editor.append_history(&config_path));
        Config {
            line_editor_config,
            config_dir,
            history_file,
        }
    }
    pub fn line_editor_config(&self) -> &rustyline::Config {
        &self.line_editor_config
    }
    pub fn history_file(&self) -> &PathBuf {
        &self.history_file
    }
}
