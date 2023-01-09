#![allow(dead_code)]

mod cexdexclient;
mod slackclient;

use std::{thread, time::Duration};

use crate::cexdexclient::client::*;
use cex_dex_monitor::{CexDexConfig, Config};

#[tokio::main]
async fn main() {
    let cfg = Config::from_yaml("secret.yaml".to_string()).unwrap();

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

    let mut awaits = Vec::new();
    for v in cfg.cex_dex_config.iter() {
        let aw = monitor(&v, &sl_client);
        awaits.push(aw);
    }

    loop {
        match awaits.pop() {
            None => break,
            Some(aw) => {
                aw.await;
            }
        }
    }
}

async fn monitor(cex_dex_cfg: &CexDexConfig, sl_client: &slackclient::client::Client) {
    let mut last_notified_state: String = String::from("");

    let cd_client = CexDexClient::new(
        cex_dex_cfg.base_url.clone(),
        cex_dex_cfg.user.clone(),
        cex_dex_cfg.pass.clone(),
    );

    println!("starting loop..., env: {}", cex_dex_cfg.env);
    loop {
        thread::sleep(Duration::from_secs(5));
        println!("\n{:#?}", chrono::Utc::now().to_rfc3339());

        let states = cd_client.get_states().await;
        if let Err(e) = states {
            println!("get states error {}", e);
            continue;
        }
        let states = states.unwrap();

        if last_notified_state.len() == 0 && states.data.len() != 0 {
            last_notified_state = states.data[states.data.len() - 1].state_id.clone();
            println!(
                "updated fisrt notified state {}\n{}",
                &last_notified_state,
                build_state_done_message(&states.data[states.data.len() - 1], &cex_dex_cfg.env)
            );
        }

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

            if let Err(e) = sl_client
                .send_message(
                    String::from("alert-virtual-taker-1"),
                    String::from(build_state_done_message(state, &cex_dex_cfg.env)),
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
    let p2_dex_token_filled = state.p2_sum_token_filled(state.token.clone());
    let p2_dex_stable_filled = state.p2_sum_token_filled(String::from("")); // if not equal -> revert amountIn & amountOut
    let p2_dex_price = if p2_dex_stable_filled == 0 as f64 {
        0.0
    } else {
        p2_dex_token_filled / p2_dex_stable_filled
    };

    format!(
        "*****
*STATE DONE*

> ENV: {}
STATE_ID: {}
SIDE: {}

P1 FILLED ORDERS: {}
P1 BASE FILLED: {}
P1 QUOTE FILLED: {}
P1 PRICE: {}

P2 FILLED ORDERS: {}
P2 BASE FILLED: {}
P2 QUOTE FILLED: {}
P2 PRICE: {}

P2 CREATED TXs: {}
P2 TOKEN FILLED: {}
P2 STABLE FILLED: {}
P2 PRICE: {}
{}

ASSET CHANGES:
{}
*****",
        env,
        state.state_id,
        state.side,
        state.count_p1_filled_orders(),
        state.p1_sum_base_filled(),
        state.p1_sum_quote_filled(),
        state.get_cex_price(&state.p1_cex_orders),
        state.count_p2_filled_orders(),
        state.p2_sum_base_filled(),
        state.p2_sum_quote_filled(),
        state.get_cex_price(&state.p2_cex_orders),
        state.p2_count_created_txs(),
        p2_dex_token_filled,
        p2_dex_token_filled,
        p2_dex_price,
        state.p2_summary_txs(),
        state.asset_changes(),
    )
}
