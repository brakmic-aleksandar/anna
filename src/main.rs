use ansi_term::Colour;
use std::{str, env, process};
use chrono::{Duration, NaiveDate, Datelike, Timelike};
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
    path.push("anna");
    path.push("anna");
    path.set_extension("toml");

    path
}

fn config() -> config::Config {
    let config_path = config_path();
    config::load_config(&config_path)
}

fn save_config(config: &config::Config) {
    let config_path = config_path();
    config::update_config(&config_path, &config);
}

fn template_path(journal_path: &Path) -> PathBuf {
    let mut template_path = PathBuf::from(journal_path);
    template_path.push("template");

    template_path
}

fn editor() -> Result<String, String> {
    let config_path = config_path();
    let config = config::load_config(&config_path);

    match config.editor {
        Some(val) => Ok(val),
        None => match env::var("EDITOR") {
            Ok(val) if !val.trim().is_empty() => Ok(val),
            _ => Err("Text editor wasn't set in config, nor EDITOR env variable was defined".to_string())
        }
    }
}

fn page_path_from_date(journal_path: &Path, date: NaiveDate) -> PathBuf {
    let config = config();
    let mut path = PathBuf::from(journal_path);
    path.push(date.year().to_string());
    path.push(format!("{:0>2}", date.month()));
    path.push(format!("{:0>2}", date.day()));
    path.set_extension(config.extension.unwrap_or("txt".into()));

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

fn today(midnight_offset: Option<u32>) -> NaiveDate {
    let offset = midnight_offset.unwrap_or(0);
    let mut today = chrono::offset::Local::now().naive_local();

    if today.hour() <= offset {
        today -= Duration::days(1);
    }

    today.date()
}

fn macros() -> HashMap<String, Box<dyn Fn() -> String>> {
    let mut config = config();

    let mut macros = HashMap::from([
        ( String::from("DATE"), Box::new(move || { today(config.midnight_offset).to_string() } ) as Box<dyn Fn() -> String> )
    ]);

    let custom_macros = config.macros.get_or_insert_with(HashMap::new);

    for m in custom_macros {
        let cmd = m.1.clone();
        macros.insert(m.0.to_string(), Box::new(move || {
            let tokens = cmd.split(" ").collect::<Vec<&str>>();
            match process::Command::new(&tokens[0])
                .args(&tokens[1..])
                .output() {
                    Ok(output) => str::from_utf8(&output.stdout).unwrap().to_string(),
                    Err(_) => stop_with_error("Failed to start custom macro")
                }
        }
        ));
    }

    macros
}

fn create_page(journal_path: &Path, page_path: &Path) {
    let template_path = template_path(journal_path);

    if template_path.exists() {

        let macros = macros();

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
        create_page(journal_path, &page_path);
    }

    edit_journal_file(journal_path, &page_path, "Page updated", offline_mode);
}

fn open_todays_page(offline_mode: bool) {
    let config = config();
    let today = today(config.midnight_offset);

    open_page(today, offline_mode);
}

fn open_page(date: NaiveDate, offline_mode: bool) {
    let config = config();

    let journal_path = match config.path {
        Some(val) => val,
        None => stop_with_error("Journal path not set in config.")
    };

    edit_page(&journal_path, date, offline_mode);
}

fn edit_template(offline_mode: bool) {
    let config = config();
    let journal_path = match config.path {
        Some(val) => val,
        None => stop_with_error("Journal path not set in config.")
    };
    let template_path = template_path(&journal_path);
    edit_journal_file(&journal_path, &template_path, "Template updated", offline_mode);
}

fn update_config(config_args: args::Config) {
    let mut config = config();
    config.editor = match config_args.editor {
        Some(editor) => Some(editor),
        None => config.editor
    };
    config.path = match config_args.path {
        Some(path) => Some(path),
        None => config.path
    };
    config.extension = match config_args.extension {
        Some(extension) => Some(extension),
        None => config.extension
    };
    config.midnight_offset = match config_args.midnight_offset {
        Some(midnight_offset) => Some(midnight_offset),
        None => config.midnight_offset
    };
    save_config(&config);
}

fn add_macro(args: args::AddMacro) {
    let mut config = config();
    let macros = config.macros.get_or_insert_with(HashMap::new);
    macros.insert(args.name, args.command);
    save_config(&config);
}

fn remove_macro(args: args::RemoveMacro) {
    let mut config = config();
    let macros = config.macros.get_or_insert_with(HashMap::new);
    macros.remove(&args.name);
    save_config(&config);
}

fn main() {
    let args = args::args();

    match args.subcommand {
        None => open_todays_page(args.offline),
        Some(args::Subcommand::Config(config)) => update_config(config),
        Some(args::Subcommand::Page(page)) => open_page(page.date, args.offline),
        Some(args::Subcommand::Template) => edit_template(args.offline),
        Some(args::Subcommand::Macro(r#macro)) => match r#macro.subcommand {
            args::MacroSubcommand::Add(add) => add_macro(add),
            args::MacroSubcommand::Rm(rm) => remove_macro(rm)
        }
    };
}
