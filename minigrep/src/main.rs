use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Parse config error: {}", err);
        process::exit(1);
    });

    println!("{} {}", config.query, config.file_path);

    run(config);
}

struct Config {
    query: String,
    file_path: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }

        Ok(Config {
            query: args[1].clone(),
            file_path: args[2].clone(),
        })
    }
}

fn run(cfg: Config) {
    let file_content = fs::read_to_string(cfg.file_path).expect("unable to read file");
    println!("file content: {}", file_content);
}

// V3
// #[derive(Debug)]
// struct Config {
//     query: String,
//     file_path: String,
// }

// fn main() {
//     let args: Vec<String> = env::args().collect();
//     let config = parse_config(&args);

//     println!("{:?}", config)
// }

// fn parse_config(args: &[String]) -> Config {
//     // using clone for faster development
//     Config {
//         query: args[1].clone(),
//         file_path: args[2].clone(),
//     }
// }

// V2
// fn main() {
//     let args: Vec<String> = env::args().collect();

//     let (query, file_path) = parse_config(&args);

//     println!("Search `{}` from `{}`", query, file_path)
// }

// fn parse_config(args: &[String]) -> (&str, &str) {
//     let query = &args[1];
//     let file_path = &args[2];

//     (query, file_path)
// }

// V1
// fn main() {
//     // Read arguments
//     let args: Vec<String> = env::args().collect();

//     // arguments length check
//     if args.len().lt(&3) {
//         panic!("expect more then 2 arguments")
//     }

//     let query = &args[1];
//     let file_path = &args[2];

//     print!("search for {} in {}\n", query, file_path);

//     // read from file
//     let file_data = fs::read_to_string(file_path).expect("Not able to read from file");

//     println!("Read from file: {}", file_data);
// }
