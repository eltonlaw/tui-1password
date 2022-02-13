/// CLI entry point
use std::io;
use tracing::{Level};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
// use tracing_subscriber::fmt::writer::MakeWriterExt;

pub mod app;
pub mod op;
pub mod utils;

pub fn run() -> Result<(), io::Error> {
    let home_dir = op::home_dir();
    // FIXME: Delete older files
    // Starts a new thread that writes to a file
    let file_appender = RollingFileAppender::new(Rotation::NEVER,
                                                 home_dir,
                                                 "run.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_max_level(Level::TRACE)
        .init();

    tracing::info!("Starting new instance");
    app::render_app();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json};

    #[test]
    fn get_in_test() {
		let v1 = json!({"a": "b", "c": {"d": "e"}});
        assert_eq!(None, utils::get_in(&v1, &vec!["e"]));
        assert_eq!(&json!({"d": "e"}), utils::get_in(&v1, &vec!["c"]).unwrap());
        assert_eq!("e", utils::get_in(&v1, &vec!["c", "d"]).unwrap());
    }

}