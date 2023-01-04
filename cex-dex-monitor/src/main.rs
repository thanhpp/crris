#![allow(dead_code)]

mod cexdexclient;
mod slackclient;

use std::{thread, time::Duration};

use crate::cexdexclient::client::*;
use cex_dex_monitor::Config;

#[tokio::main]
async fn main() {
    monitor().await;
}

async fn monitor() {
    let mut last_notified_state: String = String::from("");
    let cfg = Config::from_yaml("secret.yaml".to_string()).unwrap();
    let cd_client = CexDexClient::new(
        cfg.cex_dex_config.base_url,
        cfg.cex_dex_config.user,
        cfg.cex_dex_config.pass,
    );
    let mut sl_client = slackclient::client::Client::new();
    match cfg.slack_client_config.webhooks {
        None => panic!("empty slack webhooks"),
        Some(wh) => {
            if wh.len() == 0 {
                panic!("empty slack webhooks")
            }
            for w in wh {
                sl_client.add_webhook(w.channel, w.webhook);
            }
        }
    }

    println!("starting loop...");
    loop {
        thread::sleep(Duration::from_secs(5));
        println!("\n{:#?}", chrono::Utc::now().to_rfc3339());

        let states = cd_client.get_states().await;
        if let Err(e) = states {
            println!("get states error {}", e);
            continue;
        }
        let states = states.unwrap();

        for i in (0..states.data.len()).rev() {
            let state = &states.data[i];
            // skips empty state
            if state.state_id.len() == 0 {
                continue;
            }
            // check latest
            if last_notified_state.eq(&state.state_id) {
                break;
            }
            // set first
            if last_notified_state.len() == 0 {
                last_notified_state = state.state_id.clone();
                println!("updated fisrt notified state {}", &last_notified_state);
            }

            if let Err(e) = sl_client
                .send_message(
                    String::from("alert-virtual-taker-1"),
                    String::from(build_state_done_message(state, &cfg.cex_dex_config.env)),
                )
                .await
            {
                println!("send slack client error {}", e);
                continue;
            }

            // update latest
            last_notified_state = state.state_id.clone();
            println!("updated last notified state {}", &last_notified_state);
        }
    }
}

fn build_state_done_message(state: &cexdexclient::dto::StateData, env: &String) -> String {
    format!(
        "*****
> ENV: {}
STATE_ID: {}
*****",
        env, state.state_id,
    )
}
