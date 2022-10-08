use serde::{Serialize, Deserialize};
use std::io::Write;
use std::fs;
use std::env;
use tracing;

use super::util;

static CONFIG_FILENAME: &str = "tui-1password.yaml";

/// Figures out where to put app specific artifacts
///
/// 1. Look in hardcoded list of env vars for a config yaml.
///     - If anything does have the file, return that filepath
/// 2. For those env vars that exist try to create the directory
///     - If successfuly created, return that filepath
pub fn get_root_dir() -> String {
    let env_vars = vec!["TUI_1PASSWORD_HOME", "XDG_CONFIG_HOME", "HOME"];
    let mut fps_existing: Vec<String> = Vec::with_capacity(env_vars.len());
    // Check that env vars exist and if they do, check if they have a config file
    for env_var in env_vars   {
        if let Ok(fp) = env::var(env_var) {
            let root_dir_fp = match env_var {
                "TUI_1PASSWORD_HOME" => fp,
                "XDG_CONFIG_HOME" => format!("{}/tui-1password", fp),
                "HOME" => format!("{}/.tui-1password", fp),
                // This will never be hit, might want to consider making env_var an enum
                _ => fp,
            };
            let cfg_fp = format!("{}/{}", root_dir_fp, CONFIG_FILENAME);
            if util::file_exists(&cfg_fp) {
                return root_dir_fp;
            }
            fps_existing.push(root_dir_fp);
        }
    }
    // If none of the env vars point somewhere with a config file, try to make the
    // directory needed
    for fp in fps_existing {
        if let Ok(_) = fs::create_dir_all(&fp) {
            return fp;
        }
    }
    panic!("Couldn't find or create home directory");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub headers: Vec<String>,
    pub root_dir: String,
    pub debug: bool,
    pub clipboard_bin: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let headers = vec![
            String::from("id"),
            String::from("title"),
            String::from("updated_at"),
        ];
        let root_dir = get_root_dir();
        let debug = true;
        let clipboard_bin = String::from("wl-copy");
        AppConfig { headers, root_dir, debug, clipboard_bin }
    }
}

// FIXME: Update so that default merges into read-in config file
// FIXME: Handling of config_fp is weird, it shouldn't be part of the yaml
impl AppConfig {
    pub fn new(root_dir: String) -> Self {
        let config_fp = format!("{}/{}", root_dir, CONFIG_FILENAME);
        tracing::info!("Attempting to read config file: {}", &config_fp);
        if fs::metadata(&config_fp).is_ok() {
            tracing::info!("Config file read in from: {}", &config_fp);
            let config_str = fs::read_to_string(&config_fp)
                .unwrap_or_else(|err| panic!("Failed to read: {} - {}", &config_fp, err));
            return serde_yaml::from_str(config_str.as_str())
                .unwrap_or_else(|err| panic!("Couldn't deserialize: {} - {}", &config_fp, err));
        } else {
            tracing::info!("No config file found generating new one: {}", &config_fp);
            let ac = AppConfig::default();
            let s = serde_yaml::to_string(&ac).expect("Failed to serialize default AppConfig");
            let mut f = fs::File::create(&config_fp)
                .unwrap_or_else(|err| panic!("Failed to create: {} - {}", &config_fp, err));
            f.write_all(s.as_bytes())
                .unwrap_or_else(|err| panic!("Failed to write contents: {} - {}", &config_fp, err));
            return ac;
        }
    }
}
