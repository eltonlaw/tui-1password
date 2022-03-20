/// Render data with TUI
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
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
use tracing;
use super::op;
use super::utils;
use std::convert::TryFrom;
use std::error;

#[derive(PartialEq)]
enum AppState {
    ItemList,
}

struct App {
    table_state: TableState,
    app_state: AppState,
    headers: Vec<Vec<String>>,
    items: Vec<Vec<String>>,
}

impl App {
    fn new(headers: Vec<Vec<String>>) -> Result<App, Box<dyn error::Error>> {
        let items_raw = op::list_items().unwrap();
        let items: Vec<Vec<String>> = items_raw
            .iter()
            .map(|item_raw| {
                let mut item = Vec::new();
                for header in &headers {
                    let val;
                    if let Some(x) = utils::get_in(item_raw, header) {
                        val = x.as_str().unwrap();
                    } else {
                        val = "";
                    }
                    item.push(String::from(val));
                }
                item
            })
            .collect();
        Ok(App {
            table_state: TableState::default(),
            app_state: AppState::ItemList,
            headers,
            items,
        })
    }
    pub fn next_item(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous_item(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if app.app_state == AppState::ItemList {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => app.next_item(),
                    KeyCode::Char('j') => app.next_item(),
                    KeyCode::Up => app.previous_item(),
                    KeyCode::Char('k') => app.previous_item(),
                    _ => {}
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    if app.app_state == AppState::ItemList {
        let rects = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(1)
            .split(f.size());

        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let normal_style = Style::default().bg(Color::Blue);
        let header_cells = app.headers
            .iter()
            .map(|h| Cell::from(Span::raw(h.join("_"))).style(Style::default().fg(Color::Red)));
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
        // FIXME: These should be calculated based on size of largest value per column and
        // use `Length` instead
        let percentage = u16::try_from(100/app.headers.len()).unwrap();
        let column_widths = vec![Constraint::Percentage(percentage); app.headers.len()];
        let t = Table::new(items)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Table"))
            .highlight_style(selected_style)
            .widths(&column_widths);
        f.render_stateful_widget(t, rects[0], &mut app.table_state);
    }
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
        vec![String::from("id")],
        vec![String::from("title")],
        vec![String::from("updated_at")],
    ];
    // create app and run it
    let app = App::new(headers)?;

    let _t = TerminalModifier::new()?;

    // FIXME: To be moved into TerminalModifier
    // setup terminal
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let res = run_app(&mut terminal, app);

    // FIXME: To be moved into TerminalModifier
    // restore terminal
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).unwrap();
    terminal.show_cursor().unwrap();

    if let Err(err) = res{
        tracing::error!("{:?}", err);
    }

    Ok(())
}
