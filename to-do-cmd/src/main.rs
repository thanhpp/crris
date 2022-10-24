use structopt::StructOpt;
mod cli;
use cli::{Action::*, CommandLineArgs};
mod task;
use task::Task;

fn main() {
    // get command line arguments
    let CommandLineArgs {
        action,
        journal_file,
    } = CommandLineArgs::from_args();

    // Unpack
    let journal_file = journal_file.expect("Failed to find journal file");

    // perform action
    match action {
        Add { task } => task::add_task(journal_file, Task::new(task)),
        List => task::list_tasks(journal_file),
        Done { position } => task::complete_task(journal_file, position),
    }
    .expect("Failed to perform action");
}
