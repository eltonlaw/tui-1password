/// Interface to 1password
use serde_json::{Value};
use std::env;
use std::error;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::process::{Command};
use std::str;
use std::time::{Duration};
use tracing;

use super::err;

// Temporary tokens from `op signin` last for 30 minutes
static OP_TOKEN_TTL: u64 = 1800;

/// True if cached token exists and created less than OP_TOKEN_TTL seconds ago
/// FIXME: Use this
pub fn is_valid_cache(path: &String) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        let token_age = metadata.modified().unwrap().elapsed().unwrap();
        return token_age < Duration::from_secs(OP_TOKEN_TTL);
    } else {
        return false;
    }
}

#[derive(Debug)]
pub struct Session {
    pub name: String,
    pub token: String,
}

impl Session {
    // Currently only takes the first entry in the token
    pub fn from_cache(path: &String) -> Self {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let mut name = String::from("");
        let mut token = String::from("");
        for line in reader.lines().map(|l| l.unwrap()) {
            // FIXME: should handle just the token as well (output when --raw used)
            if line.contains("export") {
                let splits: Vec<&str> = line.split("=").collect();
                assert_eq!(2, splits.len());
                // strip out the "export "
                name = String::from(&(splits[0])[7..]);
                // strip out the quotes surrounding the token
                token = String::from(&(splits[1])[1..splits[1].len() - 1]);
            }
        }
        Session { name, token }
    }
}

/// Get directory where logs and local cache is stored
pub fn home_dir() -> String {
    // FIXME: Make sure this exists
    format!("{}/.tui-1password", env::var("HOME").unwrap())
}

// FIXME: Should cache this
pub fn get_session() -> Result<Session, Box<dyn error::Error>> {
    let op_token_path = format!("{}/token", home_dir());
    if is_valid_cache(&op_token_path) {
        let s = Session::from_cache(&op_token_path);
        tracing::info!("Started new session: {} {:?}", &op_token_path, s);
        return Ok(s);
    } else {
        tracing::error!("Failed to started new session, invalid 1password token {} ", &op_token_path);
        return Err(err::InvalidSessionError{ token: op_token_path }.into());
    }
}

// FIXME: Refresh session
// pub fn refresh_session() {}
// FIXME: if the tui is open
// pub fn auto_refresh_session() {}


// FIXME: instead of using serde_json::Error, use enum that also can be `Box<dyn error::Error>`
pub fn list_items() -> Result<Vec<Value>, serde_json::Error> {
    let session = get_session().unwrap();
    let output = Command::new("op")
                         .env(session.name, session.token)
                         .arg("item")
                         .arg("list")
                         .arg("--format=json")
                         .output().unwrap();
    let items = str::from_utf8(&output.stdout).unwrap();

    serde_json::from_str(items)

}
