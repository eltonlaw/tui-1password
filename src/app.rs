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
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};
use super::op;
use super::utils;

struct App<'a> {
    state: TableState,
    headers: Vec<Vec<&'a str>>,
    items: Vec<Vec<&'a str>>,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        // FIXME: "borrowed value does not live long enough"
        let headers = vec![
            vec!["overview", "title"],
            // vec!["overview", "website"],
            vec!["overview", "url"],
            vec!["overview", "ainfo"],
            // vec!["templateUuid"],
            // vec!["updatedAt"]
        ];
        let values = utils::serde_json_value_to_vec(op::list_items().unwrap()).unwrap();
        let items: Vec<Vec<&'a str>> = values
            .iter()
            .map(|v| vec![
                utils::get_in(v, &headers[0]).unwrap().as_str().unwrap(),
                utils::get_in(v, &headers[1]).unwrap().as_str().unwrap(),
                utils::get_in(v, &headers[2]).unwrap().as_str().unwrap(),
            ])
            .collect();
        // let items = vec![
        //     vec!["Row11", "Row12", "Row13"],
        //     vec!["Row21", "Row22", "Row23"],
        //     vec!["Row31", "Row32", "Row33"],
        //     vec!["Row41", "Row42", "Row43"],
        //     vec!["Row51", "Row52", "Row53"],
        //     vec!["Row61", "Row62\nTest", "Row63"],
        //     vec!["Row71", "Row72", "Row73"],
        //     vec!["Row81", "Row82", "Row83"],
        //     vec!["Row91", "Row92", "Row93"],
        //     vec!["Row101", "Row102", "Row103"],
        //     vec!["Row111", "Row112", "Row113"],
        //     vec!["Row121", "Row122", "Row123"],
        //     vec!["Row131", "Row132", "Row133"],
        //     vec!["Row141", "Row142", "Row143"],
        //     vec!["Row151", "Row152", "Row153"],
        //     vec!["Row161", "Row162", "Row163"],
        //     vec!["Row171", "Row172", "Row173"],
        //     vec!["Row181", "Row182", "Row183"],
        //     vec!["Row191", "Row192", "Row193"],
        // ];
        App {
            state: TableState::default(),
            headers: headers,
            items: items,
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

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
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
    let rows = app.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(*c));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    let t = Table::new(rows)
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
    let t = TerminalModifier::new()?;

    // FIXME: To be moved into TerminalModifier
    // setup terminal
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

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
