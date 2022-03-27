use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tracing;
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

pub struct TerminalModifier {
    pub terminal: Terminal<CrosstermBackend<io::Stdout>>
}

impl TerminalModifier {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        enable_raw_mode()?;
        tracing::info!("Enabled terminal raw mode");
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();
        Ok(TerminalModifier {terminal})
    }
}

impl Drop for TerminalModifier {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        tracing::info!("Disabled terminal raw mode");
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).unwrap();
        self.terminal.show_cursor().unwrap();
    }
}
