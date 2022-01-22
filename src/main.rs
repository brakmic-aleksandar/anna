use ansi_term::Colour;
use std::{env, process};
use chrono::{NaiveDate, Datelike};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

mod args;
mod config;
mod repo;
mod preprocessor;

fn stop_with_error(error_msg: &str) -> ! {
    print_err(error_msg);
    process::exit(1);
}

fn print_err(msg: &str) {
    eprintln!("{}{}", Colour::Red.paint("error: ").to_string(), msg);
}

fn print_warning(msg: &str) {
    eprintln!("{}{}", Colour::Yellow.paint("warning: ").to_string(), msg);
}

fn print_msg(msg: &str) {
    println!("{}", msg);
}

fn config_path() -> PathBuf {
    let mut path = env::home_dir().unwrap();
    path.push(".config");
    path.push("journal_cli");
    path.push("journal_cli");
    path.set_extension("toml");

    path
}

fn template_path(journal_path: &Path) -> PathBuf {
    let mut template_path = PathBuf::from(journal_path);
    template_path.push("template");
    template_path.set_extension("txt");

    template_path
}

fn editor() -> Result<String, String> {
    let config_path = config_path();
    let config = config::load_config(config_path.as_path());

    match config.editor {
        Some(val) => Ok(val),
        None => match env::var("EDITOR") {
            Ok(val) if !val.trim().is_empty() => Ok(val),
            _ => Err("Text editor wasn't set in config, nor EDITOR env variable was defined".to_string())
        }
    }
}

fn page_path_from_date(journal_path: &Path, date: NaiveDate) -> PathBuf {
    let mut path = PathBuf::from(journal_path);
    path.push(date.year().to_string());
    path.push(date.month().to_string());
    path.push(date.day().to_string());
    path.set_extension("txt");

    path
}

fn pull_changes(repo: &repo::Repo) {
    if repo.pull().is_err() {
        print_warning("Failed to pull data from remote, using local old data.");
    }
}

fn push_file(repo: &repo::Repo, path: &Path, msg: &str) {
    if repo.add(path).is_err() {
        stop_with_error("Failed to add changes.");
    }
    if repo.commit(msg).is_err() {
        stop_with_error("Failed to commit file.")
    }
    if repo.push().is_err() {
        stop_with_error("Failed to push changes.")
    }
}

fn edit_file(path: &Path) {
    let editor = match editor() {
        Ok(val) => val,
        Err(msg) => stop_with_error(&msg)
    };

    match process::Command::new(editor)
        .arg(path)
        .status() {
            Ok(_) => (),
            Err(_) => stop_with_error("Failed to start editor")
        }
}

fn edit_journal_file(journal_path: &Path, path: &Path, commit_msg: &str, offline_mode: bool) {
    let repo = repo::Repo::new(journal_path);

    if !offline_mode {
        pull_changes(&repo);
    }
    edit_file(path);
    if !offline_mode {
        push_file(&repo, path, commit_msg);
    }
}

fn create_page(journal_path: &Path, page_path: &Path) {
    let template_path = template_path(journal_path);

    if template_path.exists() {

        let macros = HashMap::from([
            (String::from("DATE"), Box::new(|| {
                chrono::offset::Local::today().naive_local().to_string()
            }))
        ]);

        let template_text = std::fs::read_to_string(template_path).unwrap();
        let processed_template = preprocessor::process(&template_text, &macros);

        let page_dir = page_path.parent().unwrap();
        if !page_dir.exists() {
            std::fs::create_dir(page_dir).unwrap();
        }
        std::fs::write(page_path, processed_template);
    }
}

fn edit_page(journal_path: &Path, page_date: NaiveDate, offline_mode: bool) {
    let page_path = page_path_from_date(journal_path, page_date);
    if !page_path.exists() {
        create_page(journal_path, page_path.as_path());
    }

    let template_path = template_path(journal_path);

    edit_journal_file(journal_path, page_path.as_path(), "Page updated", offline_mode);
}

fn open_todays_page(offline_mode: bool) {
    let config_path = config_path();
    let config = config::load_config(config_path.as_path());

    let today = chrono::offset::Local::today().naive_local();
    let journal_path = match config.path {
        Some(val) => val,
        None => stop_with_error("Journal path not set in config.")
    };

    edit_page(journal_path.as_path(), today, offline_mode);
}


fn open_page(date: NaiveDate, offline_mode: bool) {
    let config_path = config_path();
    let config = config::load_config(config_path.as_path());

    let journal_path = match config.path {
        Some(val) => val,
        None => stop_with_error("Journal path not set in config.")
    };

    edit_page(journal_path.as_path(), date, offline_mode);
}

fn edit_template(offline_mode: bool) {
    let config_path = config_path();
    let config = config::load_config(config_path.as_path());

    let journal_path = match config.path {
        Some(val) => val,
        None => stop_with_error("Journal path not set in config.")
    };
    let template_path = template_path(journal_path.as_path());

    edit_journal_file(journal_path.as_path(), template_path.as_path(), "Template updated", offline_mode);
}

fn update_config(config_args: args::Config) {
    let config_path = config_path();
    let mut config: config::Config = config::load_config(config_path.as_path());
    config.editor = match config_args.editor {
        Some(editor) => Some(editor),
        None => config.editor
    };
    config.path = match config_args.path {
        Some(path) => Some(path),
        None => config.path
    };
    config::update_config(config_path.as_path(), &config);
}

fn main() {
    let args = args::args();

    match args.action {
        None => open_todays_page(args.offline),
        Some(args::Action::Config(config)) => update_config(config),
        Some(args::Action::Page(page)) => open_page(page.date, args.offline),
        Some(args::Action::Template) => edit_template(args.offline)
    };
}
