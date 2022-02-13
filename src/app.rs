/// Render data with TUI
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};
use super::op;
use super::utils;

struct App {
    state: TableState,
    headers: Vec<Vec<String>>,
    items: Vec<Vec<String>>,
}

impl App {
    fn new(headers: Vec<Vec<String>>) -> App {
        let items_raw = op::list_items().unwrap();
        let items: Vec<Vec<String>> = items_raw
            .iter()
            .map(|item_raw| {
                let mut item = Vec::new();
                for header in &headers {
                    // let val = utils::get_in(item_raw, header).unwrap().as_str())
                    let val;
                    if let Some(x) = utils::get_in(item_raw, header) {
                        val = x.as_str().unwrap();
                    } else {
                        val = "";
                    }
                    item.push(String::from(val));
                }
                item
                // FIXME: use all headers
                // String::from(utils::get_in(v, &headers[1]).unwrap().as_str().unwrap()),
                // String::from(utils::get_in(v, &headers[2]).unwrap().as_str().unwrap()),
            })
            .collect();
        App {
            state: TableState::default(),
            headers,
            items,
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(5)
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["Header1", "Header2", "Header3"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let items = app.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(Span::raw(c)));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    let t = Table::new(items)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Table"))
        .highlight_style(selected_style)
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);
    f.render_stateful_widget(t, rects[0], &mut app.state);
}

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

pub fn render_app() -> Result<(), Box<dyn Error>> {
    // To be taken from CLI
    let headers = vec![
        vec![String::from("overview"), String::from("title")],
        vec![String::from("overview"), String::from("url")],
        vec![String::from("overview"), String::from("ainfo")],
    ];

    let _t = TerminalModifier::new()?;

    // FIXME: To be moved into TerminalModifier
    // setup terminal
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // create app and run it
    let app = App::new(headers);
    run_app(&mut terminal, app);

    // FIXME: To be moved into TerminalModifier
    // restore terminal
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).unwrap();
    terminal.show_cursor().unwrap();

    Ok(())
}
