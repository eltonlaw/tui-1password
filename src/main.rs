/// CLI entry point
use std::error::Error;
use tui_1password;

fn main() -> Result<(), Box<dyn Error>> {
    tui_1password::run()
}
