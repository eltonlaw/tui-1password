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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppView {
    ItemListView,
    ItemView,
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

// Used by `sort_item_list`
pub struct SortConfig {
    pub sort_direction: SortDirection,
    pub header: String,
}

#[derive(Debug)]
pub struct SearchState {
    pub pattern: String,
    pub match_idxs: Vec<usize>,
    pub selected_match_idx: Option<usize>,
}

pub struct App {
    pub is_running: bool,
    pub item_table_state: TableState,
    pub item_list_table_state: TableState,
    pub item_list_sort_config: SortConfig,
    pub app_view: AppView,
    pub headers: Vec<String>,
    pub items: Vec<op::ItemListEntry>,
    pub item_details: Option<op::ItemDetails>,
    pub session: op::Session,
    pub input_mode: InputMode,
    pub cmd_input: String,
    pub search_state: Option<SearchState>,
    pub clipboard_bin: String,
}

impl App {
    pub fn new(config: AppConfig) -> Result<App, Box<dyn error::Error>> {
        Ok(App {
            is_running: true,
            item_table_state: TableState::default(),
            item_list_table_state: TableState::default(),
            item_list_sort_config: SortConfig {
                header: String::from("title"),
                sort_direction: SortDirection::Ascending
            },
            app_view: AppView::ItemListView,
            headers: config.headers,
            items: Vec::new(),
            item_details: None,
            session: op::Session::new(format!("{}/token", config.root_dir))?,
            input_mode: InputMode::Normal,
            cmd_input: String::from(""),
            search_state: None,
            clipboard_bin: config.clipboard_bin,
        })
    }

    pub fn populate_items(&mut self) {
        match self.session.list_items() {
            Ok(mut items) => {
                for item in items.iter_mut() {
                    item.gen_index_term();
                }
                self.items = items;
            },
            Err(err) => {
                tracing::error!("Couldn't populate items: {}", err);
                panic!("Couldn't populate items: {}", err);
            }
        }
        self.sort_item_list();
    }

    pub fn sort_item_list(&mut self) {
        self.items.sort_by(
            // FIXME: Would be good to write a macro so that we can create comp functions for every
            // property in the ItemListEntry
            // FIXME: Create sort index so that we just need to compare sort index (would also
            // allow multi column sorting)
            match self.item_list_sort_config.header.as_str() {
                "id" => match self.item_list_sort_config.sort_direction {
                    SortDirection::Ascending  => |a: &op::ItemListEntry, b: &op::ItemListEntry| a.id.cmp(&b.id),
                    SortDirection::Descending => |a: &op::ItemListEntry, b: &op::ItemListEntry| b.id.cmp(&a.id),
                },
                "title" => match self.item_list_sort_config.sort_direction {
                    SortDirection::Ascending  => |a: &op::ItemListEntry, b: &op::ItemListEntry| a.title.to_lowercase().cmp(&b.title.to_lowercase()),
                    SortDirection::Descending => |a: &op::ItemListEntry, b: &op::ItemListEntry| b.title.to_lowercase().cmp(&a.title.to_lowercase()),
                },
                "updated_at" => match self.item_list_sort_config.sort_direction {
                    SortDirection::Ascending  => |a: &op::ItemListEntry, b: &op::ItemListEntry| a.updated_at.cmp(&b.updated_at),
                    SortDirection::Descending => |a: &op::ItemListEntry, b: &op::ItemListEntry| b.updated_at.cmp(&a.updated_at),
                },
                &_           => |a: &op::ItemListEntry, b: &op::ItemListEntry| a.title.to_lowercase().cmp(&b.title.to_lowercase()),
            }
        );
    }

    /// Return the index of the last element for the passed in app view
    fn table_max_index(&self, app_view: &AppView) -> usize {
        let len = match app_view {
            AppView::ItemListView => self.items.len(),
            AppView::ItemView => self.item_details.as_ref().unwrap().fields.len(),
        };
        // Hacky just-in-case bit for always returning a usize. There should
        // always be at least one item in the list of items so `else` should
        // never be hit.
        return if len == 0 { 0 } else { len - 1 }
    }

    /// Returns index of selected row for passed in AppView
    fn selected_index(&self, app_view: &AppView) -> usize {
        match app_view {
            AppView::ItemListView => self.item_list_table_state.selected().unwrap_or(0),
            AppView::ItemView => self.item_table_state.selected().unwrap_or(0),
        }
    }

    /// Set index of selected row to i for passed in AppView
    fn set_selected_index(&mut self, i: i32, app_view: &AppView) {
        let max_i = self.table_max_index(&app_view);
        let table_state = match app_view {
            AppView::ItemListView => &mut self.item_list_table_state,
            AppView::ItemView => &mut self.item_table_state,
        };
        let selected_i: usize = {
            if i < 0 {
                0
            } else if i > max_i as i32 {
                max_i
            } else {
                usize::try_from(i).unwrap()
            }
        };
        table_state.select(Some(selected_i));
    }

    /// Add some int to selected row index for passed in AppView.
    /// Pass in positive i_delta to go down a row, pass in negative i_delta
    /// to go up a row.
    fn add_selected_index(&mut self, i_delta: i32, app_view: &AppView) {
        let i_current = i32::try_from(self.selected_index(&app_view)).unwrap();
        self.set_selected_index(i_current + i_delta, &app_view);
    }

    fn current_item(&self) -> &op::ItemListEntry {
        &self.items[self.selected_index(&AppView::ItemListView)]
    }

    fn current_item_detail(&self) -> &op::ItemDetailsField {
        let i = self.selected_index(&AppView::ItemView);
        &self.item_details.as_ref().unwrap().fields[i]
    }

    fn reset_cmd_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.cmd_input = String::from("");
    }

    fn select_next_search(&mut self) {
        if let Some(ref mut search_state) = self.search_state {
            if !search_state.match_idxs.is_empty() {
                let new_mi = match search_state.selected_match_idx {
                    Some(mi) => util::inc_or_wrap(mi, search_state.match_idxs.len()),
                    // FIXME: Instead of going to top search, find closest to current
                    None => 0,
                };
                let new_selected_i = search_state.match_idxs[new_mi];
                search_state.selected_match_idx = Some(new_mi);
                self.set_selected_index(i32::try_from(new_selected_i).unwrap(),
                                        &AppView::ItemListView);
            }
        }
    }

    fn run_command(&mut self) {
        let components: Vec<&str> = self.cmd_input.split(" ").collect();
        let n_args = components.len();
        // From https://github.com/rust-lang/rust/issues/59159 immutable references
        // have to all be finished with by the time the mutable reference is used
        let arg0 = String::from(*components.get(0).unwrap_or(&""));
        let arg1 = String::from(*components.get(1).unwrap_or(&""));
        let arg2 = String::from(*components.get(2).unwrap_or(&""));

        let mut chars = arg0.chars();
        let ch = chars.next().unwrap();
        let cmd = chars.as_str();
        match ch {
            ':' => match cmd {
                "e" => self.populate_items(),
                "q" => self.is_running = false,
                "qa" => self.is_running = false,
                "sort" => {
                    self.item_list_sort_config = SortConfig {
                        header: arg1,
                        sort_direction: {
                            if n_args == 2 {
                                SortDirection::Ascending
                            } else {
                                match arg2.as_str() {
                                    "asc" => SortDirection::Ascending,
                                    "desc" => SortDirection::Descending,
                                    &_ => SortDirection::Ascending,
                                }
                            }
                        }
                    };
                    self.sort_item_list();
                },
                _ => {}
            },
            '/' => {
                self.select_next_search();
            }
            _ => {}
        };
    }

    fn yank(&self) {
        let s = match self.app_view {
            AppView::ItemListView => self.current_item().title.as_str(),
            AppView::ItemView => {
                self.current_item_detail().value.as_ref().unwrap().as_str()
            },
        };
        let cmd_components: Vec<&str> = self.clipboard_bin.as_str().split(" ").collect();
        let mut cmd = Command::new(cmd_components[0]);
        if cmd_components.len() > 1 {
            cmd.args(&cmd_components[1..]);
        }
        let process = match cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn() {
            Err(why) => {
                tracing::error!("Couldn't spawn {:?}: {}", cmd_components, why);
                panic!("Couldn't spawn {}: {}", self.clipboard_bin, why);
            },
            Ok(process) => process,
        };
        match process.stdin.unwrap().write_all(s.as_bytes()) {
            Err(why) => panic!("Couldn't write to {} stdin: {}", self.clipboard_bin, why),
            Ok(_) => {},
        }
    }

    fn populate_item_details(&mut self) {
        // FIXME: Set to None if unwrap fails and amend things that grab `item_details`
        // need to handle it too
        let mut item_details = self.session.get_item(&self.current_item().id).unwrap();
        item_details.fill_none_fields();
        self.item_details = Some(item_details);
    }

    fn search_item_list(&mut self) {
        if let Some(ref mut search_state) = self.search_state {
            // FIXME: Filtering could filter over the previous
            // generated search_state.match_idxs so string matching
            // doesn't need to run on every element every time
            // (probably want to generate some lookup table once here)
            search_state.match_idxs = self.items.iter()
                .enumerate()
                .filter(|(_, ile)| {
                    // FIXME: Use regex for case insensitive
                    ile.has_pattern(search_state.pattern.as_str())
                })
                .map(|(i, _)| i)
                .collect();
        }
        tracing::info!(
            "Counts={:?} SearchState={:?}",
            self.search_state.as_ref().unwrap().match_idxs,
            &self.search_state);
    }

    fn enter_command_mode(&mut self, cmd_input: &str) {
        self.input_mode = InputMode::Command;
        self.cmd_input = String::from(cmd_input);
        if cmd_input == "/" {
            self.search_state = Some(SearchState {
                pattern: String::from(""),
                match_idxs: Vec::new(),
                selected_match_idx: None,
            });
        }
    }

    /// Currently only handles KeyEvents, modifies app state based on inputs
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key_event) => match self.input_mode {
                InputMode::Normal => match self.app_view {
                    AppView::ItemListView => match key_event.code {
                        KeyCode::Char('d') => match key_event.modifiers {
                            // FIXME: This should dynamically go halfway
                            KeyModifiers::CONTROL => self.add_selected_index(6, &AppView::ItemListView),
                            _ => {}
                        }
                        KeyCode::Char('u') => match key_event.modifiers {
                            // FIXME: This should dynamically go halfway
                            KeyModifiers::CONTROL => self.add_selected_index(-6, &AppView::ItemListView),
                            _ => {}
                        }
                        KeyCode::Char('q') => self.is_running = false,
                        KeyCode::Down      => self.add_selected_index(1, &AppView::ItemListView),
                        KeyCode::Char('j') => self.add_selected_index(1, &AppView::ItemListView),
                        KeyCode::Up        => self.add_selected_index(-1, &AppView::ItemListView),
                        KeyCode::Char('k') => self.add_selected_index(-1, &AppView::ItemListView),
                        KeyCode::Char('G') => self.set_selected_index(self.table_max_index(&AppView::ItemListView) as i32, &AppView::ItemListView),
                        // FIXME: vim uses `gg` as the "go to first element" key binding
                        KeyCode::Char('g') => self.item_list_table_state.select(Some(0)),
                        KeyCode::Char('n') => self.select_next_search(),
                        KeyCode::Char(':') => self.enter_command_mode(":"),
                        KeyCode::Char('/') => self.enter_command_mode("/"),
                        KeyCode::Char('y') => self.yank(),
                        KeyCode::Enter     => {
                            self.populate_item_details();
                            self.app_view = AppView::ItemView;
                        },
                        KeyCode::Char('R') => self.populate_items(),
                        _ => {}
                    },
                    AppView::ItemView => match key_event.code {
                        KeyCode::Char('d') => match key_event.modifiers {
                            // FIXME: This should dynamically go halfway
                            KeyModifiers::CONTROL => self.add_selected_index(6, &AppView::ItemView),
                            _ => {}
                        }
                        KeyCode::Char('u') => match key_event.modifiers {
                            // FIXME: This should dynamically go halfway
                            KeyModifiers::CONTROL => self.add_selected_index(-6, &AppView::ItemView),
                            _ => {}
                        }
                        KeyCode::Char('q') => self.app_view = AppView::ItemListView,
                        KeyCode::Char('R') => self.populate_item_details(),
                        KeyCode::Down      => self.add_selected_index(1, &AppView::ItemView),
                        KeyCode::Char('j') => self.add_selected_index(1, &AppView::ItemView),
                        KeyCode::Up        => self.add_selected_index(-1, &AppView::ItemView),
                        KeyCode::Char('k') => self.add_selected_index(-1, &AppView::ItemView),
                        KeyCode::Char('y') => self.yank(),
                        _ => {}
                    },
                },
                InputMode::Command => match key_event.code {
                    KeyCode::Enter => {
                        self.run_command();
                        self.reset_cmd_input();
                    },
                    KeyCode::Char(c) => {
                        self.cmd_input.push(c);
                        if self.cmd_input.starts_with('/') {
                            // unwrap() is fine to use in this case cause SearchState will have
                            // been created in the entry to InputMode::Command
                            if let Some(ref mut s) = self.search_state {
                                s.pattern.push(c.to_ascii_lowercase());
                            }
                            self.search_item_list();
                        }
                    },
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
