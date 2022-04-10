/// Render data with TUI
use crossterm::{
    event::{self, Event, KeyCode},
};
use std::cmp;
use std::convert::TryFrom;
use std::env;
use std::error;
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
use super::op;
use super::terminal;

#[derive(PartialEq)]
enum AppState {
    ItemListView,
    ItemView,
}

struct App {
    table_state: TableState,
    app_state: AppState,
    headers: Vec<String>,
    items: Vec<op::ItemListEntry>,
    session: op::Session,
}

/// Get directory where logs and local cache is stored
pub fn home_dir() -> String {
    // FIXME: Make sure this exists
    format!("{}/.tui-1password", env::var("HOME").unwrap())
}

impl App {
    fn new(headers: Vec<String>) -> Result<App, Box<dyn error::Error>> {
        let items: Vec<op::ItemListEntry> = Vec::new();
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

    pub fn current_item(&self) -> &op::ItemListEntry {
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

/// Given a vec of column display names, return a tui Row object
fn new_header_row<'a>(headers: &'a Vec<String>) -> Row<'a> {
    let header_cells = headers
        .iter()
        .map(|h| Cell::from(Span::raw(h)).style(Style::default().fg(Color::Red)));
    Row::new(header_cells)
        .style(Style::default().bg(Color::Blue))
        .height(1)
        .bottom_margin(1)
}

fn new_item_list_row<'a, 'b>(item: &'a op::ItemListEntry, headers: &'b Vec<String>) -> Row<'a> {
    let mut height = 1;
    let cells = headers.iter().map(|header| {
        let val = match header.as_str() {
            "id" => &item.id,
            "title" => &item.title,
            "updated_at" => &item.updated_at,
            _ => "",
        };
        height = cmp::max(height, val.chars().filter(|c| *c == '\n').count());
        Cell::from(Span::raw(val))
    });
    Row::new(cells).height(height as u16).bottom_margin(1)
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(1)
        .split(f.size());

    if app.app_state == AppState::ItemListView {
        let table_items = app.items.iter().map(|item| {
            new_item_list_row(&item, &app.headers)
        });
        // FIXME: These should be calculated based on size of largest value per column and
        // use `Length` instead
        let percentage = u16::try_from(100/app.headers.len()).unwrap();
        let column_widths = vec![Constraint::Percentage(percentage); app.headers.len()];
        let t = Table::new(table_items)
            .header(new_header_row(&app.headers))
            .block(Block::default().borders(Borders::ALL).title("Table"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .widths(&column_widths);
        f.render_stateful_widget(t, rects[0], &mut app.table_state);
    } else if app.app_state == AppState::ItemView {
        let item_detail_headers = vec![String::from("field"), String::from("value")];
        let item_details = app.session.get_item(&app.current_item().id).unwrap();
        let table_items = item_details.fields
            .iter()
            .filter(|field| { field.value.is_some() && field.label.is_some() })
            .map(|field| {
                Row::new(vec![
                    Cell::from(Span::raw(field.label.as_ref().unwrap())),
                    Cell::from(Span::raw(field.value.as_ref().unwrap()))
                ])
            });
        let column_widths = vec![Constraint::Percentage(50); 2];
        let t = Table::new(table_items)
            .header(new_header_row(&item_detail_headers))
            .block(Block::default().borders(Borders::ALL).title("Entry"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .widths(&column_widths);
        f.render_stateful_widget(t, rects[0], &mut app.table_state);
    }
}

pub fn render_app() -> Result<(), Box<dyn Error>> {
    // To be taken from CLI
    let headers = vec![
        String::from("id"),
        String::from("title"),
        String::from("updated_at"),
    ];
    // create app and run it
    match App::new(headers) {
        Result::Ok(mut app) => {
            app.populate_items();

            let mut tm = terminal::TerminalModifier::new()?;
            let res = run_app(&mut tm.terminal, app);

            if let Err(err) = res{
                tracing::error!("{:?}", err);
            }
        }
        Result::Err(err) => eprintln!("{}", err),
    };
    Ok(())
}
