/// Render data with TUI
use crossterm::{
    event::{Event, KeyCode},
};
use std::convert::TryFrom;
use std::error;
use tui::{
    backend::{Backend},
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};
use super::app_config::{AppConfig};
use super::op;
use super::ui;

/// Different available views that the app can display API data
///
/// - ItemListView: for looking through the list of stored data
/// - ItemView: display all details of specific item
/// - Exit: When entered, stops the app
#[derive(PartialEq)]
pub enum AppView {
    ItemListView,
    ItemView,
    Exit,
}

/// Normal mode is regular operation, command is when `:` is typed
#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Command,
    // FIXME:
    // Edit,
}

pub struct App {
    pub table_state: TableState,
    pub app_view: AppView,
    pub headers: Vec<String>,
    pub items: Vec<op::ItemListEntry>,
    pub session: op::Session,
    pub input_mode: InputMode,
    pub cmd_input: String,
}

pub fn try_inc(idx: Option<usize>, max: usize) -> usize {
    match idx {
        Some(i) => {
            if i >= max - 1 {
                0
            } else {
                i + 1
            }
        }
        None => 0,
    }
}

pub fn try_dec(idx: Option<usize>, max: usize) -> usize {
    match idx {
        Some(i) => {
            match i {
                0 => max - 1,
                _ => i - 1,
            }
        }
        None => 0,
    }
}

impl App {
    pub fn new(config: AppConfig) -> Result<App, Box<dyn error::Error>> {
        let items: Vec<op::ItemListEntry> = Vec::new();
        Ok(App {
            table_state: TableState::default(),
            app_view: AppView::ItemListView,
            headers: config.headers,
            items,
            session: op::Session::new(config.token_path)?,
            input_mode: InputMode::Normal,
            cmd_input: String::from(":"),
        })
    }
    pub fn populate_items(&mut self) {
        self.items = self.session.list_items().unwrap();
    }

    // FIXME: Sort by sort key
    pub fn sort_items(&mut self) {
        self.items.sort_by(|a, b| a.title.cmp(&b.title));
    }

    pub fn next_item(&mut self) {
        let i = try_inc(self.table_state.selected(), self.items.len());
        self.table_state.select(Some(i));
    }

    pub fn previous_item(&mut self) {
        let i = try_dec(self.table_state.selected(), self.items.len());
        self.table_state.select(Some(i));
    }

    pub fn current_item(&self) -> &op::ItemListEntry {
        let i = self.table_state.selected().unwrap_or(0);
        &self.items[i]
    }

    fn reset_cmd_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.cmd_input = String::from(":");
    }

    fn run_command(&mut self) {
        match self.cmd_input.as_str() {
            ":q" => self.app_view = AppView::Exit,
            ":qa" => self.app_view = AppView::Exit,
            _ => {}
        }
    }

    /// Currently only handles KeyEvents, modifies app state based on inputs
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key_event) => match self.input_mode {
                InputMode::Normal => match self.app_view {
                    AppView::ItemListView => match key_event.code {
                        KeyCode::Char('q') => self.app_view = AppView::Exit,
                        KeyCode::Down      => self.next_item(),
                        KeyCode::Char('j') => self.next_item(),
                        KeyCode::Up        => self.previous_item(),
                        KeyCode::Char('k') => self.previous_item(),
                        KeyCode::Char(':') => self.input_mode = InputMode::Command,
                        KeyCode::Enter     => self.app_view = AppView::ItemView,
                        _ => {}
                    },
                    AppView::ItemView => match key_event.code {
                        KeyCode::Char('q') => self.app_view = AppView::ItemListView,
                        _ => {}
                    },
                    AppView::Exit => {},
                },
                InputMode::Command => match key_event.code {
                    KeyCode::Enter => { self.run_command(); self.reset_cmd_input(); },
                    KeyCode::Char(c) => self.cmd_input.push(c),
                    KeyCode::Backspace => { self.cmd_input.pop(); },
                    KeyCode::Esc => self.reset_cmd_input(),
                    _ => {},
                }
            },
            _ => {}
        }
    }
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let constraints = match app.input_mode {
        InputMode::Normal => vec![Constraint::Percentage(100)],
        InputMode::Command => vec![Constraint::Min(1), Constraint::Length(1)],
    };
    let chunks = Layout::default()
        .constraints(constraints)
        .margin(1)
        .split(f.size());

    if app.app_view == AppView::ItemListView {
        app.sort_items();
        let table_items = app.items
            .iter()
            .map(|item| {
                ui::new_item_list_row(&item, &app.headers)
            });
        // FIXME: These should be calculated based on size of largest value per column and
        // use `Length` instead
        let percentage = u16::try_from(100/app.headers.len()).unwrap();
        let column_widths = vec![Constraint::Percentage(percentage); app.headers.len()];
        let t = Table::new(table_items)
            .header(ui::new_header_row(&app.headers))
            .block(Block::default().borders(Borders::NONE).title("Table"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .widths(&column_widths);
        f.render_stateful_widget(t, chunks[0], &mut app.table_state);
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
            .block(Block::default().borders(Borders::NONE).title("Entry"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .widths(&column_widths);
        f.render_stateful_widget(t, chunks[0], &mut app.table_state);
    }
    if app.input_mode == InputMode::Command {
        let input = Paragraph::new(app.cmd_input.as_ref());
        f.render_widget(input, chunks[1]);
    }
}
