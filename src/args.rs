use clap::Parser;
use chrono::NaiveDate;

#[derive(clap::Args, Debug)]
pub struct Config {
    /// Repo path
    #[clap(long)]
    pub path: Option<std::path::PathBuf>,

    /// Default text editor
    #[clap(long)]
    pub editor: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct Page {
    #[clap(long)]
    pub date: NaiveDate,
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Edit config
    Config(Config),
    /// Edit specific page
    Page(Page),
    /// Edit template
    Template
}

/// Simple git based journaling app
#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Args {
    /// Edit files in offline mode, without pushing them
    #[clap(long, short)]
    pub offline: bool,

    #[clap(subcommand)]
    pub action: Option<Action>,
}

pub fn args() -> Args {
    Args::parse()
}
