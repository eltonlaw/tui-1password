use crossterm::{
    execute, terminal::{disable_raw_mode, enable_raw_mode},
};
use tracing;
use std::{error::Error, io};

pub struct TerminalModifier {}

impl TerminalModifier {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        enable_raw_mode()?;
        tracing::info!("Enabled terminal raw mode");
        Ok(TerminalModifier {})
    }
}

impl Drop for TerminalModifier {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        tracing::info!("Disabled terminal raw mode");
    }
}
