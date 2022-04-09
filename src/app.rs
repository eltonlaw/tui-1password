/// Render data with TUI
use crossterm::{
    event::{self, Event, KeyCode},
};
use std::cmp;
use std::env;
use std::{error::Error, io};
use tui::{
    backend::{Backend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};
use tracing;
use serde_json::{Value};
use super::op;
use super::utils;
use super::terminal;
use std::convert::TryFrom;
use std::error;

#[derive(PartialEq)]
enum AppState {
    ItemListView,
    ItemView,
}

struct App {
    table_state: TableState,
    app_state: AppState,
    headers: Vec<Vec<String>>,
    items: Vec<Value>,
    session: op::Session,
}

/// Get directory where logs and local cache is stored
pub fn home_dir() -> String {
    // FIXME: Make sure this exists
    format!("{}/.tui-1password", env::var("HOME").unwrap())
}

impl App {
    fn new(headers: Vec<Vec<String>>) -> Result<App, Box<dyn error::Error>> {
        let items: Vec<Value> = Vec::new();
        let op_token_path = format!("{}/token", home_dir());
        Ok(App {
            table_state: TableState::default(),
            app_state: AppState::ItemListView,
            headers,
            items,
            session: op::Session::new(op_token_path)?,
        })
    }
    pub fn populate_items(&mut self) {
        self.items = self.session.list_items().unwrap();
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

    pub fn current_item(&self) -> &Value {
        let i = self.table_state.selected().unwrap_or(0);
        &self.items[i]
    }

    pub fn change_app_state(&mut self, new_app_state: AppState) {
        self.app_state = new_app_state;
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if app.app_state == AppState::ItemListView {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down      => app.next_item(),
                    KeyCode::Char('j') => app.next_item(),
                    KeyCode::Up        => app.previous_item(),
                    KeyCode::Char('k') => app.previous_item(),
                    KeyCode::Enter     => app.change_app_state(AppState::ItemView),
                    _ => {}
                }
            } else if app.app_state == AppState::ItemView {
                match key.code {
                    KeyCode::Char('q') => app.change_app_state(AppState::ItemListView),
                    _ => {}
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(1)
        .split(f.size());
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);

    if app.app_state == AppState::ItemListView {
        let header_cells = app.headers
            .iter()
            .map(|h| Cell::from(Span::raw(h.join("_"))).style(Style::default().fg(Color::Red)));
        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1)
            .bottom_margin(1);

        let table_items = app.items.iter().map(|item| {
            let mut height = 1;
            let cells = app.headers.iter().map(|header| {
                let val;
                if let Some(x) = utils::get_in(item, header) {
                    val = x.as_str().unwrap();
                    height = cmp::max(height, val.chars().filter(|c| *c == '\n').count());
                } else {
                    val = "";
                }
                Cell::from(Span::raw(val))
            });
            Row::new(cells).height(height as u16).bottom_margin(1)
        });
        // FIXME: These should be calculated based on size of largest value per column and
        // use `Length` instead
        let percentage = u16::try_from(100/app.headers.len()).unwrap();
        let column_widths = vec![Constraint::Percentage(percentage); app.headers.len()];
        let t = Table::new(table_items)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Table"))
            .highlight_style(selected_style)
            .widths(&column_widths);
        f.render_stateful_widget(t, rects[0], &mut app.table_state);
    } else if app.app_state == AppState::ItemView {
        let header_cells = ["field", "value"]
            .iter()
            .map(|h| Cell::from(Span::raw(h.to_string())).style(Style::default().fg(Color::Red)));
        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1)
            .bottom_margin(1);
        let table_items = vec![
            Row::new(vec![Cell::from(Span::raw("field1")), Cell::from(Span::raw("value"))]),
            Row::new(vec![Cell::from(Span::raw("field2")), Cell::from(Span::raw("value"))]),
            Row::new(vec![Cell::from(Span::raw("field3")), Cell::from(Span::raw("value"))]),
        ];
        let column_widths = vec![Constraint::Percentage(50); 2];
        let t = Table::new(table_items)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Entry"))
            .highlight_style(selected_style)
            .widths(&column_widths);
        f.render_stateful_widget(t, rects[0], &mut app.table_state);
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
    let mut app = App::new(headers)?;
    app.populate_items();

    let mut tm = terminal::TerminalModifier::new()?;
    let res = run_app(&mut tm.terminal, app);

    if let Err(err) = res{
        tracing::error!("{:?}", err);
    }

    Ok(())
}
