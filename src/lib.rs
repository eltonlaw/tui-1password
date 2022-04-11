/// CLI entry point
use crossterm::event::{self, Event, KeyCode};
use std::io;
use std::error::Error;
use tracing::{Level};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tui::{
    backend::{Backend},
    Terminal};

pub mod app;
pub mod err;
pub mod op;
pub mod terminal;
pub mod ui;

fn draw_app<B: Backend>(terminal: &mut Terminal<B>, mut app: app::App) -> io::Result<()> {
    loop {
        terminal.draw(|f| app::ui(f, &mut app))?;

        app.handle_event(event::read()?);
        if app.app_view == app::AppView::Exit {
            return Ok(());
        }
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let home_dir = app::home_dir();
    // FIXME: Delete older files.
    let file_appender = RollingFileAppender::new(Rotation::NEVER, home_dir, "run.log");
    // Starts a new thread that writes to a file
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_max_level(Level::TRACE)
        .init();

    tracing::info!("Starting new instance tui-1password instance");
    // To be taken from CLI
    let headers = vec![
        String::from("id"),
        String::from("title"),
        String::from("updated_at"),
    ];
    // create app and run it
    match app::App::new(headers) {
        Result::Ok(mut app) => {
            app.populate_items();

            let mut tm = terminal::TerminalModifier::new()?;

            // Loop forever, if return, there's an error
            let res = draw_app(&mut tm.terminal, app);
            if let Err(err) = res{
                tracing::error!("{:?}", err);
            }
        }
        Result::Err(err) => eprintln!("{}", err),
    };
    Ok(())
}
