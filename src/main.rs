use std::io;
use termion::raw::IntoRawMode;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Block, Borders, Widget};
use tui::layout::{Layout, Constraint, Direction};

// Render once and quit
fn main() -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // Create a surrounding border
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(80),
                    Constraint::Percentage(10)
                ].as_ref()
            )
            .split(f.size());
        let block = Block::default()
             .title("Block")
             .borders(Borders::ALL);
        f.render_widget(block, chunks[0]);
        let block = Block::default()
             .title("Block 2")
             .borders(Borders::ALL);
        f.render_widget(block, chunks[1]);
        let block = Block::default()
             .title("Block 3")
             .borders(Borders::ALL);
        f.render_widget(block, chunks[2]);
    })?;
    Ok(())
}
