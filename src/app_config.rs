use std::env;

#[derive(Debug)]
pub struct AppConfig {
    pub headers: Vec<String>,
    pub home_dir: String,
    pub token_path: String,
    pub is_debug: bool,
}

impl AppConfig {
    pub fn new() -> Self {
        let headers = vec![
            String::from("id"),
            String::from("title"),
            String::from("updated_at"),
        ];
        let home_dir = format!("{}/.tui-1password", env::var("HOME").unwrap());
        let token_path = format!("{}/token", home_dir); 
        let is_debug = true;
        AppConfig { headers, home_dir, token_path, is_debug }
    }
}
