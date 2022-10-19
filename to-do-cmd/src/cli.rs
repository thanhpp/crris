use std::path::PathBuf;
use structopt::StructOpt;

/// derive(StructOpt) & structopt(...) instruct Rust to generate a cmd argument parser using the CommandLineArgs struct
///

#[derive(Debug, StructOpt)]
pub enum Action {
    Add {
        #[structopt()]
        task: String,
    },
    Done {
        #[structopt()]
        position: usize,
    },
    List,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Journal", about = "command line to-do app")]
pub struct CommandLineArgs {
    #[structopt(subcommand)]
    pub action: Action,
    #[structopt(parse(from_os_str), short, long)]
    pub journal_file: Option<PathBuf>,
}
