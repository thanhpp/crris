#![allow(dead_code)]

mod cexdexclient;

use crate::cexdexclient::client::*;
use cex_dex_monitor::Config;

#[tokio::main]
async fn main() {
    let cfg = Config::from_yaml("secret.yaml".to_string())
        .unwrap()
        .cex_dex_config;

    let client = CexDexClient::new(cfg.base_url, cfg.user, cfg.pass);

    let resp = client.get_states().await;
    if let Err(e) = resp {
        println!("get_states error {}", e);
        return;
    }

    println!("response\n{:#?}", resp.unwrap());
}
