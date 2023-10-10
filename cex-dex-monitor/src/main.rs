#![allow(dead_code)]

mod balance_monitor;
mod cexdexclient;
mod slackclient;

use std::{
    collections::{HashMap, HashSet},
    thread,
    time::Duration,
};

use crate::cexdexclient::client::*;
use cex_dex_monitor::{CexDexConfig, Config};

#[tokio::main]
async fn main() {
    let cfg = Config::from_yaml("secret.yaml".to_string()).unwrap();

    let mut sl_client = slackclient::client::Client::new();
    match cfg.slack_client_config.webhooks {
        None => panic!("empty slack webhooks"),
        Some(wh) => {
            if wh.is_empty() {
                panic!("empty slack webhooks")
            }
            for w in wh {
                sl_client.add_webhook(w.channel, w.webhook);
            }
        }
    }

    for v in cfg.cex_dex_config {
        let sl_client_con_1 = sl_client.clone();
        let sl_client_con_2 = sl_client.clone();
        let v_1 = v.clone();
        let v_2 = v.clone();
        tokio::spawn(async move {
            monitor(&v_1, sl_client_con_1).await;
        });

        if v.env.ne("prod") {
            continue;
        }

        tokio::spawn(async move {
            monitor_balances(&v_2, sl_client_con_2).await;
        });
    }

    // block for ctrl + C
    loop {
        thread::sleep(Duration::from_secs(10))
    }
}

async fn monitor(cex_dex_cfg: &CexDexConfig, sl_client: slackclient::client::Client) {
    let mut done_states: HashSet<String> = HashSet::new();
    let mut_done_states = &mut done_states;

    let cd_client = CexDexClient::new(
        cex_dex_cfg.base_url.clone(),
        cex_dex_cfg.user.clone(),
        cex_dex_cfg.pass.clone(),
    );

    println!("starting loop..., env: {}", cex_dex_cfg.env);
    loop {
        thread::sleep(Duration::from_secs(
            10 + (chrono::Utc::now().timestamp() % 10) as u64,
        ));
        let now = chrono::Utc::now().to_rfc3339();

        let states = cd_client.get_filled_done_states().await;
        if let Err(e) = states {
            println!("{} get states error {}", now, e);
            continue;
        }
        let states = states.unwrap();

        // insert first time run
        if mut_done_states.is_empty() {
            for i in 0..states.data.len() {
                let state = &states.data[i];
                // skips empty state
                if state.state_id.is_empty() {
                    continue;
                }
                // insert states
                mut_done_states.insert(state.state_id.clone());
            }
            println!(
                "{} first run: inserted {} states",
                now,
                mut_done_states.len()
            );
            continue;
        }

        for i in 0..states.data.len() {
            let state = &states.data[i];
            // skips empty state
            if state.state_id.is_empty() {
                continue;
            }

            // check if exist
            if !mut_done_states.insert(state.state_id.clone()) {
                continue;
            }

            // notify new state
            if let Err(e) = sl_client
                .send_message(
                    String::from("alert-virtual-taker-1"),
                    build_state_done_message(state, &cex_dex_cfg.env),
                )
                .await
            {
                println!("{} send slack client error {}", now, e);
                continue;
            }
            println!(
                "{} env {}, send update of state id {}",
                now, cex_dex_cfg.env, state.state_id
            );
        }

        // remove old done states - is not in states
        let mut remove_state_ids: Vec<String> = Vec::new();
        for v in mut_done_states.iter() {
            let mut should_remove = true;
            for i in 0..states.data.len() {
                let state = &states.data[i];
                if v.eq(&state.state_id) {
                    should_remove = false;
                    break;
                }
            }
            if should_remove {
                remove_state_ids.push(v.clone());
            }
        }

        for id in remove_state_ids {
            mut_done_states.remove(&id);
            println!("{} env {}, removed state id {}", now, cex_dex_cfg.env, id);
        }
    }
}

fn build_state_done_message(state: &cexdexclient::dto::StateData, env: &String) -> String {
    let p2_dex_token_filled = state.p2_sum_token_filled(&state.token);
    let p2_dex_stable_filled = state.p2_sum_token_filled(&String::from("USDT")); // now using usdt only
    let p2_dex_price = if p2_dex_token_filled == 0 as f64 {
        0.0
    } else {
        p2_dex_stable_filled / p2_dex_token_filled
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
        p2_dex_stable_filled,
        p2_dex_price,
        state.p2_summary_txs(),
        state.asset_changes(),
    )
}

async fn monitor_balances(cex_dex_cfg: &CexDexConfig, sl_client: slackclient::client::Client) {
    println!("monitor_balances started");

    let cd_client = CexDexClient::new(
        cex_dex_cfg.base_url.clone(),
        cex_dex_cfg.user.clone(),
        cex_dex_cfg.pass.clone(),
    );
    let epsilon = 0.00001;

    let mut s =
        balance_monitor::Service::new(cex_dex_cfg.env.clone(), epsilon, cd_client, &sl_client);

    s.monitor_balance().await;
}
