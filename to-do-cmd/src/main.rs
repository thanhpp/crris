use std::path::PathBuf;

use anyhow::{anyhow, Ok};
use structopt::StructOpt;
mod cli;
use cli::{Action::*, CommandLineArgs};
mod task;
use task::Task;

fn main() -> anyhow::Result<()> {
    // get command line arguments
    let CommandLineArgs {
        action,
        journal_file,
    } = CommandLineArgs::from_args();

    // Unpack
    let journal_file = journal_file
        .or_else(find_default_journal_file)
        .ok_or(anyhow!("Failed to find journal file"))?;

    // perform action
    match action {
        Add { task } => task::add_task(journal_file, Task::new(task)),
        List => task::list_tasks(journal_file),
        Done { position } => task::complete_task(journal_file, position),
    }
    .expect("Failed to perform action");

    Ok(())
}

fn find_default_journal_file() -> Option<PathBuf> {
    home::home_dir().map(|mut path| {
        path.push(".rusty_journal.json");
        path
    })
}
