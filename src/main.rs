/// CLI entry point
use std::io;
use tracing::{Level};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::writer::MakeWriterExt;

mod app;
mod op;


fn main() -> Result<(), io::Error> {
    let home_dir = op::home_dir();

    // FIXME: Delete older files
    let file_appender = RollingFileAppender::new(Rotation::NEVER, home_dir, "run.log"); 
	let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
	tracing_subscriber::fmt()
		.with_writer(non_blocking)
		.with_ansi(false)
		.with_max_level(Level::TRACE)
		.init();

    tracing::info!(target: "tui-1password", "Starting new instance");
    tracing::info!(target: "tui-1password", "Started new session: {:?}", op::get_session().unwrap());
    app::render_app();
    Ok(())
}
