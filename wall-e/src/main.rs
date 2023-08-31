use tokio::signal::unix::{signal, SignalKind};

mod config;
mod gg_sheet;
mod tailscale;
mod tele_handler;

#[tokio::main]
async fn main() {
    // cfg
    let cfg = config::MainConfig::new("./secret.yaml").expect("parse main config");

    // test tailscale
    let c = tailscale::Client::new(
        cfg.tailscale_config.auth.clone(),
        cfg.tailscale_config.org.clone(),
    );

    // init google sheet
    let ggs_client =
        gg_sheet::GgsClient::new(&cfg.google_secret_file, &cfg.add_balance_config.sheet_id)
            .await
            .expect("set up ggs client error");

    // init telegram
    tele_handler::TeleHandler::start(cfg, ggs_client, c)
        .await
        .expect("start tele handler");

    wait_for_signal().await;
}

// wait_for_signal: https://blog.logrocket.com/guide-signal-handling-rust/
async fn wait_for_signal() {
    let mut sigint = signal(SignalKind::interrupt()).expect("init signal interrupt error");

    match sigint.recv().await {
        Some(()) => println!("SIGINT received"),
        None => println!("sth went wrong"),
    }
}
