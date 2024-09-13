use std::{io::stderr, process::exit};

use clap::Parser;
use mdbook_header_footer::{run, App, Command};
use tracing::debug;
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    init_tracing();
    let app = App::parse();
    if let Some(Command::Supports { renderer }) = app.command {
        debug!(renderer, "Supports");
        exit(0)
    }
    run()
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_writer(stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
