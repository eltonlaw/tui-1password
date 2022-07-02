/// Render data with TUI
use crossterm::{
    event::{Event, KeyCode, KeyModifiers},
};
use std::convert::TryFrom;
use std::io::Write;
use std::process::{Command, Stdio};
use std::error;
use tui::{
    backend::{Backend},
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::Span, widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};
use super::app_config::{AppConfig};
use super::op;
use super::ui;
use super::util;

/// Different available views that the app can display API data
///
/// - ItemListView: for looking through the list of stored data
/// - ItemView: display all details of specific item
/// - Exit: When entered, stops the app
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

pub struct App {
    pub item_table_state: TableState,
    pub item_list_table_state: TableState,
    pub app_view: AppView,
    pub headers: Vec<String>,
    pub items: Vec<op::ItemListEntry>,
    pub item_details: Option<op::ItemDetails>,
    pub session: op::Session,
    pub input_mode: InputMode,
    pub cmd_input: String,
    pub clipboard_bin: String,
}

impl App {
    pub fn new(config: AppConfig) -> Result<App, Box<dyn error::Error>> {
        Ok(App {
            item_table_state: TableState::default(),
            item_list_table_state: TableState::default(),
            app_view: AppView::ItemListView,
            headers: config.headers,
            items: Vec::new(),
            item_details: None,
            session: op::Session::new(format!("{}/token", config.root_dir))?,
            input_mode: InputMode::Normal,
            cmd_input: String::from(""),
            clipboard_bin: config.clipboard_bin,
        })
    }

    pub fn populate_items(&mut self) {
        self.items = self.session.list_items().unwrap();
    }

    pub fn sort_item_list_by(&mut self, header: String, sd: SortDirection) {
        self.items.sort_by(
            match header.as_str() {
                "id"         => match sd {
                    SortDirection::Ascending  => |a: &op::ItemListEntry, b: &op::ItemListEntry| a.id.cmp(&b.id),
                    SortDirection::Descending => |a: &op::ItemListEntry, b: &op::ItemListEntry| b.id.cmp(&a.id),
                },
                "title"      => match sd {
                    SortDirection::Ascending  => |a: &op::ItemListEntry, b: &op::ItemListEntry| a.title.cmp(&b.title),
                    SortDirection::Descending => |a: &op::ItemListEntry, b: &op::ItemListEntry| b.title.cmp(&a.title),
                },
                "updated_at" => match sd {
                    SortDirection::Ascending  => |a: &op::ItemListEntry, b: &op::ItemListEntry| a.updated_at.cmp(&b.updated_at),
                    SortDirection::Descending => |a: &op::ItemListEntry, b: &op::ItemListEntry| b.updated_at.cmp(&a.updated_at),
                },
                &_           => |a: &op::ItemListEntry, b: &op::ItemListEntry| a.title.cmp(&b.title),
            }
        );
    }

    fn next_item(&mut self, n: u32, app_view: AppView) {
        for _ in 0..n {
            match app_view {
                AppView::ItemListView => {
                    let i = util::inc_or_wrap(self.item_list_table_state.selected(), self.items.len());
                    self.item_list_table_state.select(Some(i));
                },
                AppView::ItemView => {
                    let i = util::inc_or_wrap(
                        self.item_table_state.selected(),
                        self.item_details.as_ref().unwrap().fields.len()
                    );
                    self.item_table_state.select(Some(i));
                },
                _ => {}
            };
        };
    }

    fn previous_item(&mut self, n: u32, app_view: AppView) {
        for _ in 0..n {
            match app_view {
                AppView::ItemListView => {
                    let i = util::dec_or_wrap(self.item_list_table_state.selected(), self.items.len());
                    self.item_list_table_state.select(Some(i));
                },
                AppView::ItemView => {
                    let i = util::dec_or_wrap(
                        self.item_table_state.selected(),
                        self.item_details.as_ref().unwrap().fields.len()
                    );
                    self.item_table_state.select(Some(i));
                },
                _ => {}
            };
        };
    }

    fn current_item(&self) -> &op::ItemListEntry {
        let i = self.item_list_table_state.selected().unwrap_or(0);
        &self.items[i]
    }

    fn current_item_detail(&self) -> &op::ItemDetailsField {
        let i = self.item_table_state.selected().unwrap_or(0);
        &self.item_details.as_ref().unwrap().fields[i]
    }

    fn reset_cmd_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.cmd_input = String::from("");
    }

    fn run_command(&mut self) {
        let components: Vec<&str> = self.cmd_input.split(" ").collect();
        let n_args = components.len();
        // From https://github.com/rust-lang/rust/issues/59159 immutable references
        // have to all be finished with by the time the mutable reference is used
        let arg0 = String::from(*components.get(0).unwrap_or(&""));
        let arg1 = String::from(*components.get(1).unwrap_or(&""));
        let arg2 = String::from(*components.get(2).unwrap_or(&""));
        match arg0.as_str() {
            ":q" => self.app_view = AppView::Exit,
            ":qa" => self.app_view = AppView::Exit,
            ":sort" => {
                self.sort_item_list_by(
                    arg1,
                    if n_args == 2 {
                        SortDirection::Ascending
                    } else {
                        match arg2.as_str() {
                            "asc" => SortDirection::Ascending,
                            "desc" => SortDirection::Descending,
                            &_ => SortDirection::Ascending,
                        }
                    },
                )
            },
            _ => {}
        }
    }

    fn yank(&self) {
        let s = match self.app_view {
            AppView::ItemListView => self.current_item().title.as_str(),
            AppView::ItemView => {
                self.current_item_detail().value.as_ref().unwrap().as_str()
            },
            _ => { "" },
        };
        let process = match Command::new(self.clipboard_bin.as_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn() {
            Err(why) => panic!("Couldn't spawn {}: {}", self.clipboard_bin, why),
            Ok(process) => process,
        };
        match process.stdin.unwrap().write_all(s.as_bytes()) {
            Err(why) => panic!("Couldn't write to {} stdin: {}", self.clipboard_bin, why),
            Ok(_) => {},
        }
    }

    fn populate_item_details(&mut self) {
        self.item_details = Some(self.session.get_item(&self.current_item().id).unwrap());
    }

    /// Currently only handles KeyEvents, modifies app state based on inputs
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key_event) => match self.input_mode {
                InputMode::Normal => match self.app_view {
                    AppView::ItemListView => match key_event.code {
                        KeyCode::Char('d') => match key_event.modifiers {
                            // FIXME: This should dynamically go halfway
                            KeyModifiers::CONTROL => self.next_item(6, AppView::ItemListView),
                            _ => {}
                        }
                        KeyCode::Char('u') => match key_event.modifiers {
                            // FIXME: This should dynamically go halfway
                            KeyModifiers::CONTROL => self.previous_item(6, AppView::ItemListView),
                            _ => {}
                        }
                        KeyCode::Char('q') => self.app_view = AppView::Exit,
                        KeyCode::Down      => self.next_item(1, AppView::ItemListView),
                        KeyCode::Char('j') => self.next_item(1, AppView::ItemListView),
                        KeyCode::Up        => self.previous_item(1, AppView::ItemListView),
                        KeyCode::Char('k') => self.previous_item(1, AppView::ItemListView),
                        KeyCode::Char(':') => {
                            self.input_mode = InputMode::Command;
                            self.cmd_input = String::from(":");
                        },
                        KeyCode::Char('/') => {
                            self.input_mode = InputMode::Command;
                            self.cmd_input = String::from("/");
                        },
                        KeyCode::Char('y') => self.yank(),
                        KeyCode::Enter     => {
                            self.populate_item_details();
                            self.app_view = AppView::ItemView;
                        },
                        _ => {}
                    },
                    AppView::ItemView => match key_event.code {
                        KeyCode::Char('d') => match key_event.modifiers {
                            // FIXME: This should dynamically go halfway
                            KeyModifiers::CONTROL => self.next_item(6, AppView::ItemView),
                            _ => {}
                        }
                        KeyCode::Char('u') => match key_event.modifiers {
                            // FIXME: This should dynamically go halfway
                            KeyModifiers::CONTROL => self.previous_item(6, AppView::ItemView),
                            _ => {}
                        }
                        KeyCode::Char('q') => self.app_view = AppView::ItemListView,
                        KeyCode::Down      => self.next_item(1, AppView::ItemView),
                        KeyCode::Char('j') => self.next_item(1, AppView::ItemView),
                        KeyCode::Up        => self.previous_item(1, AppView::ItemView),
                        KeyCode::Char('k') => self.previous_item(1, AppView::ItemView),
                        KeyCode::Char('y') => self.yank(),
                        _ => {}
                    },
                    AppView::Exit => {},
                },
                InputMode::Command => match key_event.code {
                    KeyCode::Enter => { self.run_command(); self.reset_cmd_input(); },
                    KeyCode::Char(c) => self.cmd_input.push(c),
                    KeyCode::Backspace => {
                        self.cmd_input.pop();
                        if self.cmd_input.is_empty() {
                            self.input_mode = InputMode::Normal;
                        }
                    },
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
        f.render_stateful_widget(t, chunks[0], &mut app.item_list_table_state);
    } else if app.app_view == AppView::ItemView {
        let item_detail_headers = vec![String::from("field"), String::from("value")];
        let table_items = app.item_details.as_ref().unwrap().fields
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
        f.render_stateful_widget(t, chunks[0], &mut app.item_table_state);
    }
    if app.input_mode == InputMode::Command {
        let input = Paragraph::new(app.cmd_input.as_ref());
        f.render_widget(input, chunks[1]);
    }
}
