use chrono::{serde::ts_seconds, DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fs::{File, OpenOptions},
    io::{Error, ErrorKind, Result, Seek, SeekFrom},
    path::PathBuf,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub text: String,

    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl Task {
    pub fn new(text: String) -> Task {
        let created_at: DateTime<Utc> = Utc::now();
        Task { text, created_at }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let created_at = self.created_at.with_timezone(&Local).format("%F %H:%M");
        write!(f, "{:<50} [{}]", self.text, created_at)
    }
}

pub fn add_task(journal_path: PathBuf, task: Task) -> Result<()> {
    // open file
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(journal_path)?;

    // read file & parse to a vector of Tasks
    let mut tasks: Vec<Task> = collect_tasks(&file)?;

    // Rewind the file
    file.seek(SeekFrom::Start(0))?;

    // write modified task back to the file
    tasks.push(task);
    serde_json::to_writer(file, &tasks)?;

    Ok(())
}

pub fn complete_task(journal_path: PathBuf, task_position: usize) -> Result<()> {
    // Open file
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(journal_path)?;

    // read tasks from file
    let mut tasks: Vec<Task> = match serde_json::from_reader(&file) {
        Ok(tasks) => tasks,
        Err(e) if e.is_eof() => Vec::new(),
        Err(e) => Err(e)?,
    };

    // remove the task from the given position
    // length check
    if tasks.len() == 0 || task_position > tasks.len() {
        return Err(Error::new(ErrorKind::InvalidInput, "Invalid Task ID"));
    }

    tasks.remove(task_position - 1);

    // rewide the file cursor
    file.seek(SeekFrom::Start(0))?;
    // truncate the file
    file.set_len(0)?;

    // write tp file
    serde_json::to_writer(file, &tasks)?;

    Ok(())
}

pub fn collect_tasks(mut file: &File) -> Result<Vec<Task>> {
    // rewind the file (to read from the start)
    file.seek(SeekFrom::Start(0))?;

    let tasks = match serde_json::from_reader(file) {
        Ok(tasks) => tasks,
        Err(e) if e.is_eof() => Vec::new(),
        Err(e) => Err(e)?,
    };

    // reset file cursor
    file.seek(SeekFrom::Start(0))?;

    Ok(tasks)
}

pub fn list_tasks(journal_path: PathBuf) -> Result<()> {
    // Open
    let file = OpenOptions::new().read(true).open(journal_path)?;

    // read tasks
    let tasks = collect_tasks(&file)?;

    if tasks.is_empty() {
        println!("Empty task list");
        return Ok(());
    }

    let mut order: u32 = 1;
    for task in tasks {
        println!("{}: {}", order, task);
        order += 1;
    }

    Ok(())
}
