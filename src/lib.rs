/// CLI entry point
use crossterm::event;
use std::io;
use std::error::Error;
use tracing::{Level};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tui::{
    backend::{Backend},
    Terminal};

pub mod app;
pub mod app_config;
pub mod err;
pub mod op;
pub mod terminal;
pub mod ui;
pub mod util;

fn draw_app<B: Backend>(terminal: &mut Terminal<B>, mut app: app::App) -> io::Result<()> {
    loop {
        terminal.draw(|f| app::ui(f, &mut app))?;

        app.handle_event(event::read()?);
        if !app.is_running {
            return Ok(());
        }
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    // To be updated with CLI/config file
    let root_dir = app_config::get_root_dir();

    // FIXME: COPIED FROM setup_tracing, start:
    let file_appender = RollingFileAppender::new(Rotation::NEVER, &root_dir, "run.log");
    // Starts a new thread that writes to a file
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_max_level(Level::TRACE)
        .init();
    // FIXME: setup_tracing END
    let config = app_config::AppConfig::new(root_dir);
    tracing::info!("Starting new instance tui-1password instance with config: {:?}", config);

    // create app and run it
    match app::App::new(config) {
        Result::Ok(mut app) => {
            app.populate_items();

            let mut tm = terminal::TerminalModifier::new()?;
            // Loop forever, if return, there's an error
            let res = draw_app(&mut tm.terminal, app);
            if let Err(err) = res {
                eprintln!("{}", err);
                tracing::error!("App loop ended and returned error: {:?}", err);
            }
        }
        Result::Err(err) => {
            eprintln!("{}", err);
            tracing::error!("Couldn't create app: {}", err);
        },
    };
    Ok(())
}
