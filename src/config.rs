use serde::{Serialize, Deserialize};
use toml;
use std::fs;
use std::path::Path;
use std::io::ErrorKind;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub editor: Option<String>,
    pub path: Option<std::path::PathBuf>
}

impl Default for Config {
    fn default() -> Config {
        Config {
            editor: None,
            path: None
        }
    }
}

fn load_config_str(path: &Path) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

pub fn load_config(path: &Path) -> Config {
    match load_config_str(path) {
        Ok(val) => toml::from_str::<Config>(&val).expect("Failed to parse config file."),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Config::default(),
            other_error => {
                panic!("Problem opening config file: {:?}", other_error)
            }
        }
    }
}

pub fn update_config(path: &std::path::Path, config: &Config) {
    let val = toml::to_string::<Config>(config).expect("Failed to serialize config.");
    let config_dir = path.parent().unwrap();
    match fs::create_dir_all(config_dir) {
        Err(e) if e.kind() != ErrorKind::AlreadyExists => panic!("Failed to create config dir: {:?}", e),
        _ => fs::write(path, val).expect("Failed to write to config file.")
    };
}
