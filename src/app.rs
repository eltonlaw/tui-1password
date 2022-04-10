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
use super::ui;

#[derive(PartialEq)]
pub enum AppView {
    ItemListView,
    ItemView,
}

pub struct App {
    pub table_state: TableState,
    pub app_view: AppView,
    pub headers: Vec<String>,
    pub items: Vec<op::ItemListEntry>,
    pub session: op::Session,
}

/// Get directory where logs and local cache is stored
pub fn home_dir() -> String {
    // FIXME: Make sure this exists
    format!("{}/.tui-1password", env::var("HOME").unwrap())
}

impl App {
    pub fn new(headers: Vec<String>) -> Result<App, Box<dyn error::Error>> {
        let items: Vec<op::ItemListEntry> = Vec::new();
        let op_token_path = format!("{}/token", home_dir());
        Ok(App {
            table_state: TableState::default(),
            app_view: AppView::ItemListView,
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

    pub fn change_app_view(&mut self, new_app_view: AppView) {
        self.app_view = new_app_view;
    }
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(1)
        .split(f.size());

    if app.app_view == AppView::ItemListView {
        let table_items = app.items.iter().map(|item| {
            ui::new_item_list_row(&item, &app.headers)
        });
        // FIXME: These should be calculated based on size of largest value per column and
        // use `Length` instead
        let percentage = u16::try_from(100/app.headers.len()).unwrap();
        let column_widths = vec![Constraint::Percentage(percentage); app.headers.len()];
        let t = Table::new(table_items)
            .header(ui::new_header_row(&app.headers))
            .block(Block::default().borders(Borders::ALL).title("Table"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .widths(&column_widths);
        f.render_stateful_widget(t, rects[0], &mut app.table_state);
    } else if app.app_view == AppView::ItemView {
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
            .header(ui::new_header_row(&item_detail_headers))
            .block(Block::default().borders(Borders::ALL).title("Entry"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .widths(&column_widths);
        f.render_stateful_widget(t, rects[0], &mut app.table_state);
    }
}
