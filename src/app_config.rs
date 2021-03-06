use serde::{Serialize, Deserialize};
use std::io::Write;
use std::fs;
use std::env;
use tracing;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub headers: Vec<String>,
    pub root_dir: String,
    pub debug: bool,
    pub clipboard_bin: String,
}

impl AppConfig {
    fn default() -> Self {
        let headers = vec![
            String::from("id"),
            String::from("title"),
            String::from("updated_at"),
        ];
        let root_dir = format!("{}/.tui-1password", env::var("HOME").unwrap());
        let debug = true;
        let clipboard_bin = String::from("wl-copy");
        AppConfig { headers, root_dir, debug, clipboard_bin }
    }
    pub fn new() -> Self {
        let config_fp = format!("{}/.tui-1password/tui-1password.yaml", env::var("HOME").unwrap());
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
