use std::fs;

use tokio::signal::unix::{signal, SignalKind};

mod telegram_handler;

const TELEGRAM_BOT_TOKEN_FILE: &str = "telegram_t_wall_e_bot_token";

#[tokio::main]
async fn main() {
    let data = fs::read_to_string(TELEGRAM_BOT_TOKEN_FILE).expect("read telegram token file error");

    tokio::spawn(async move { telegram_handler::start(&data).await });

    wait_for_signal().await
}

// wait_for_signal: https://blog.logrocket.com/guide-signal-handling-rust/
async fn wait_for_signal() {
    let mut sigint = signal(SignalKind::interrupt()).expect("init signal interrupt error");

    match sigint.recv().await {
        Some(()) => println!("SIGINT received"),
        None => println!("sth went wrong"),
    }
}
