use std::fs;
use std::fs::File;
use std::error::Error;
use std::io::{BufReader, BufRead};
use std::time::{Duration};
use std::process::Command;

// Temporary tokens from `op signin` last for 30 minutes
static OP_TOKEN_TTL: u64 = 1800;

// FIXME: is_valid_cache is only about the outer file existing and not expired
// not the contents, maybe call this `is_valid_cache_metadata` or
// `is_valid_cache_file`, `is_not_expired`. Naming 2 sounds a bit better, naming
// 3 is probably most accurate. Can decide on this later
/// True if cached token exists and
pub fn is_valid_cache(path: &str) -> bool {
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
    pub fn from_cache(path: &str) -> Self {
		let file = File::open(path).unwrap();
		let reader = BufReader::new(file);
		let mut name = String::from("");
		let mut token = String::from("");
		for line in reader.lines().map(|l| l.unwrap()) {
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

pub fn get_session(path: &str) -> Result<Session, Box<dyn Error>> {
    println!("{:?}", is_valid_cache(path));
    return Ok(Session::from_cache(path));
}

pub fn list_items() {
    let output = Command::new("op")
                         // .env()
                         .arg("list")
                         .arg("items")
                         .output();
}
