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

    /// Number of hours after midnight before new starts
    #[clap(long)]
    pub midnight_offset: Option<u32>,

    /// File extension for journal entries
    #[clap(long)]
    pub extension: Option<String>
}

#[derive(clap::Args, Debug)]
pub struct Page {
    #[clap(long)]
    pub date: NaiveDate,
}

#[derive(clap::Args, Debug)]
pub struct Macro {
    #[clap(subcommand)]
    pub subcommand: MacroSubcommand
}

#[derive(clap::Subcommand, Debug)]
pub enum MacroSubcommand {
    /// Add new macro
    Add(AddMacro),
    /// Remove macro
    Rm(RemoveMacro)
}

#[derive(clap::Args, Debug)]
pub struct AddMacro {
    #[clap()]
    pub name: String,
    pub command: String
}

#[derive(clap::Args, Debug)]
pub struct RemoveMacro {
    #[clap()]
    pub name: String
}

#[derive(clap::Subcommand, Debug)]
pub enum Subcommand {
    /// Edit config
    Config(Config),
    /// Edit specific page
    Page(Page),
    /// Edit template
    Template,
    /// Add or remove macro
    Macro(Macro)
}

/// Simple git based journaling app
#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Args {
    /// Edit files in offline mode, without pushing them
    #[clap(long, short)]
    pub offline: bool,

    #[clap(subcommand)]
    pub subcommand: Option<Subcommand>,
}

pub fn args() -> Args {
    Args::parse()
}
