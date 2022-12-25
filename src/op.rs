/// Interface to 1password
use serde::{Deserialize, Serialize};
use serde_json;
use std::error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufRead, Write};
use std::process::{Command, Stdio};
use std::str;
use std::time::Duration;
use tracing;
use rpassword;

use super::err;

// Temporary tokens from `op signin` last for 30 minutes
const OP_TOKEN_TTL: u64 = 1800;

#[derive(Debug)]
pub struct Session {
    pub name: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemUrl {
    label: Option<String>,
    primary: Option<bool>,
    href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDetailsVault {
    pub id: String,
    pub name: String,
}

/// Struct representing each element in the json list returned by `op item list`
#[derive(Debug, Serialize, Deserialize)]
pub struct ItemListEntry {
    pub id: String,
    pub title: String,
    pub version: u8,
    pub vault: ItemDetailsVault,
    // FIXME: Should be an enum
    pub category: String,

    // FIXME: should be date-time
    pub last_edited_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub additional_information: Option<String>,
    pub urls: Option<Vec<ItemUrl>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDetailsField {
    pub id: Option<String>,
    pub r#type: Option<String>,
    pub purpose: Option<String>,
    pub label: Option<String>,
    pub value: Option<String>
}

/// Struct representing the json map returned by `op item get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDetails {
    pub id: String,
    pub title: String,
    pub version: u8,
    // FIXME: Should be an enum
    pub category: String,

    // FIXME: should be date-time
    pub last_edited_by: String,
    pub created_at: String,
    pub updated_at: String,

    pub vault: ItemDetailsVault,
    pub fields: Vec<ItemDetailsField>,
}

impl ItemDetails {
    pub fn fill_none_fields(&mut self) {
		for field in self.fields.iter_mut() {
			match field.value {
				Some(_) => {},
				None => {
                    field.value = Some(String::from(""));
                },
			}
		}
    }
}

impl Session {
    fn signin(token_path: &String) -> Result<(),io::Error> {
		let token_file = File::create(token_path).unwrap();
		let token_stdio = Stdio::from(token_file);
        let password = rpassword::prompt_password("Enter your 1password master password: ").unwrap();
        match Command::new("op")
            .arg("signin")
            .arg("-f")
            .stdin(Stdio::piped())
            .stdout(token_stdio)
            .spawn()
        {
            Ok(mut child) => {
                child.stdin.as_ref().unwrap().write(password.as_bytes()).unwrap();
                child.wait().unwrap();
                Ok(())
            },
            Err(e) => {
                Err(e)
            }
        }
    }
    /// True if cached token exists and created less than OP_TOKEN_TTL seconds ago
    pub fn is_active_token_file(path: &String) -> bool {
        if let Ok(metadata) = fs::metadata(path) {
            let token_age = metadata.modified().unwrap().elapsed().unwrap();
            return token_age < Duration::from_secs(OP_TOKEN_TTL);
        } else {
            return false;
        }
    }
    // Currently only takes the first entry in the token
    pub fn from_token_file(path: &String) -> Result<Self, Box<dyn error::Error>> {
        let file = File::open(path)?;
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
        if name == "" && token == "" {
            return Err(err::InvalidSessionError{
                msg: String::from("op failed to sign in. Please check your password and try again")
            }.into());
        } else {
            return Ok(Session { name, token });
        }
    }
    pub fn new(token_path: String) -> Result<Self, Box<dyn error::Error>> {
        let res = Session::from_token_file(&token_path);
        if res.is_ok() {
            if Session::is_active_token_file(&token_path) {
                return res;
            } else {
                println!("Valid token found but expired, please sign in again");
            }
        }
        Session::signin(&token_path)?;
        return Session::from_token_file(&token_path);
    }

    // FIXME: Refresh session
    // pub fn refresh_session() {}
    // FIXME: if the tui is open
    // pub fn auto_refresh_session() {}

    // FIXME: instead of using serde_json::Error, use enum that also can be `Box<dyn error::Error>`
    pub fn list_items(&self) -> Result<Vec<ItemListEntry>, serde_json::Error> {
        let output = Command::new("op")
                             .env(&self.name, &self.token)
                             .arg("item")
                             .arg("list")
                             .arg("--format=json")
                             .arg("--cache")
                             .output().unwrap();
        let items = str::from_utf8(&output.stdout).unwrap();

        serde_json::from_str(items)

    }

    pub fn get_item(&self, item_name: &String) -> Result<ItemDetails, serde_json::Error> {
        let output = Command::new("op")
                             .env(&self.name, &self.token)
                             .arg("item")
                             .arg("get")
                             .arg(item_name)
                             .arg("--format=json")
                             .arg("--cache")
                             .output().unwrap();
        let items = str::from_utf8(&output.stdout).unwrap();

        serde_json::from_str(items)

    }
}
