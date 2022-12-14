use std::error::Error;
use std::{env, fs};

pub struct Config {
    query: String,
    file_path: String,
    ignore_case: bool,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }

        Ok(Config {
            query: args[1].clone(),
            file_path: args[2].clone(),
            ignore_case: env::var("IGNORE_CASE").is_ok(),
        })
    }

    // build: receive an iterator as input
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next(); // skip the first element

        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("query string not found"),
        };

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("file path not found"),
        };

        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            query: query,
            file_path: file_path,
            ignore_case: ignore_case,
        })
    }
}

pub fn run(cfg: Config) -> Result<(), Box<dyn Error>> {
    let file_content = fs::read_to_string(cfg.file_path)?;
    println!("file content: {}", file_content);

    let result = if cfg.ignore_case {
        search_case_insensitive(&cfg.query, &file_content)
    } else {
        search(&cfg.query, &file_content)
    };

    for line in result {
        println!("{line}");
    }

    Ok(())
}

fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    // let mut result = Vec::new();

    // for line in contents.lines() {
    //     if line.contains(query) {
    //         result.push(line);
    //     }
    // }

    // result

    // V2: use iterator
    contents
        .lines()
        .filter(|line| line.contains(query))
        .collect()
}

pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    let mut result = Vec::new();

    for line in contents.lines() {
        if line.to_lowercase().contains(&query) {
            result.push(line);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
";
        assert_eq!(vec!["safe, fast, productive."], search(query, contents))
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";
        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents),
        )
    }
}
