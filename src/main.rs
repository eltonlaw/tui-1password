/// CLI entry point
use std::io;
use tui_1password;

fn main() -> Result<(), io::Error> {
    tui_1password::run()
}
