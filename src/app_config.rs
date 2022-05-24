use std::env;

#[derive(Debug)]
pub struct AppConfig {
    pub headers: Vec<String>,
    pub root_dir: String,
    pub is_debug: bool,
    pub clipboard_bin: String,
}

impl AppConfig {
    pub fn new() -> Self {
        let headers = vec![
            String::from("id"),
            String::from("title"),
            String::from("updated_at"),
        ];
        let root_dir = format!("{}/.tui-1password", env::var("HOME").unwrap());
        let is_debug = true;
        let clipboard_bin = String::from("wl-copy");
        AppConfig { headers, root_dir, is_debug, clipboard_bin }
    }
}
