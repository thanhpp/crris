use std::fs;

use tokio::signal::unix::{signal, SignalKind};

mod gg_sheet;
mod telegram_handler;

const TELEGRAM_BOT_TOKEN_FILE: &str = "telegram_t_wall_e_bot_token";
const GOOGLE_SECRET_FILE: &str = "/home/thanhpp/.secrets/ggs_private_key.json";
const GOOGLE_SHEET_ID: &str = "1MKqvQ4tQiw0pk5LFlqW3CcZpOTzH5k8r6W9cwCSS1u8";

#[tokio::main]
async fn main() {
    // init google sheet
    let ggs_client = gg_sheet::GgsClient::new(GOOGLE_SECRET_FILE, GOOGLE_SHEET_ID)
        .await
        .expect("set up ggs client error");

    ggs_client
        .read_range("Sheet1!D2")
        .await
        .expect("read range error");

    let r = ggs_client
        .find_empty_row("Sheet1!A:A")
        .await
        .expect("find empty range error");
    println!("row {}", r);

    // init telegram
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
