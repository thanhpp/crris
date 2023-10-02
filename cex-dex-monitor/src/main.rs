#![allow(dead_code)]

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
        thread::sleep(Duration::from_secs(5));
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

    let mut yesterday_balances: HashMap<String, f64> = HashMap::new();
    let mut yesterday_balance_update: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
    let mut last_balances: HashMap<String, f64> = HashMap::<String, f64>::new();
    let mut last_balance_update: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
    let cd_client = CexDexClient::new(
        cex_dex_cfg.base_url.clone(),
        cex_dex_cfg.user.clone(),
        cex_dex_cfg.pass.clone(),
    );
    let short_noti_dur = chrono::Duration::hours(1);
    let long_noti_dur = chrono::Duration::hours(24);
    let interval_sleep = Duration::from_secs(60 * 10);
    let epsilon = 0.00001;

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;

        let utc_now = chrono::Utc::now();
        if !last_balances.is_empty() && utc_now - last_balance_update < short_noti_dur {
            continue;
        }

        let (curr_balances, is_rebalancing) = match get_balance(&cd_client).await {
            Ok(r) => r,
            Err(e) => {
                println!("get balance error: {}", e);
                continue;
            }
        };
        if is_rebalancing {
            continue;
        }
        if last_balances.is_empty() {
            last_balances = curr_balances.clone();
            last_balance_update = chrono::Utc::now();
            yesterday_balances = curr_balances;
            yesterday_balance_update = chrono::Utc::now();
            if let Err(e) = send_balances_msg(&sl_client, &cex_dex_cfg.env, &last_balances).await {
                println!("send balance error: {}", e);
            }
            continue;
        }

        // short interval diff
        let diff_vec = calculate_diff(&last_balances, &curr_balances, epsilon);

        if !diff_vec.is_empty() {
            last_balances = curr_balances.clone();
            last_balance_update = utc_now;

            if let Err(e) = send_diff_msg(
                &cex_dex_cfg.env,
                &last_balance_update,
                &utc_now,
                &diff_vec,
                &sl_client,
            )
            .await
            {
                println!("send message error: {}", e);
            }
        }

        // long interval diff
        if utc_now - yesterday_balance_update >= long_noti_dur {
            if let Err(e) =
                send_balances_msg(&sl_client, &cex_dex_cfg.env, &yesterday_balances).await
            {
                println!("send balance error: {}", e);
            }

            yesterday_balances = curr_balances;
            yesterday_balance_update = chrono::Utc::now();
        }

        // 10 mins
        tokio::time::sleep(interval_sleep).await;
    }
}

async fn get_balance(
    cd_client: &cexdexclient::client::CexDexClient,
) -> anyhow::Result<(HashMap<String, f64>, bool)> {
    let cex_balances = cd_client.get_cex_balanace().await?;
    let dex_balances = cd_client.get_dex_balanace().await?;

    let mut curr_balance = HashMap::<String, f64>::new();
    for (asset, b) in cex_balances.data.balances.iter() {
        match curr_balance.get_mut(asset) {
            Some(asset_b) => *asset_b += b.free + b.locked,
            None => {
                curr_balance.insert(asset.clone(), b.free + b.locked);
            }
        }
    }

    for (asset, b) in dex_balances.data.balances.iter() {
        match curr_balance.get_mut(asset) {
            Some(asset_b) => *asset_b += b,
            None => {
                curr_balance.insert(asset.clone(), *b);
            }
        }
    }

    for (asset, b) in dex_balances.data.contract_balances.iter() {
        match curr_balance.get_mut(asset) {
            Some(asset_b) => *asset_b += b,
            None => {
                curr_balance.insert(asset.clone(), *b);
            }
        }
    }

    Ok((
        curr_balance,
        cex_balances.data.is_rebalancing || dex_balances.data.is_rebalancing,
    ))
}

fn calculate_diff(
    last_balances: &HashMap<String, f64>,
    curr_balances: &HashMap<String, f64>,
    epsilon: f64,
) -> Vec<(String, f64)> {
    let mut diff_map = curr_balances.clone();

    for (k, v) in last_balances.iter() {
        match diff_map.get_mut(k) {
            None => {
                diff_map.insert(k.clone(), -*v);
            }
            Some(b) => *b -= *v,
        }
    }

    let mut diff_vec = diff_map
        .drain()
        .filter(|(_, v)| v.abs() >= epsilon)
        .map(|(k, v)| (k, v))
        .collect::<Vec<(String, f64)>>();
    diff_vec.sort_unstable_by(|a, b| a.0.cmp(&b.0));

    diff_vec
}

async fn send_diff_msg(
    env: &str,
    last_balance_update: &chrono::DateTime<chrono::Utc>,
    utc_now: &chrono::DateTime<chrono::Utc>,
    diff_vec: &[(String, f64)],
    sl_client: &slackclient::client::Client,
) -> anyhow::Result<()> {
    let mut msg = format!(
        "*****
*ASSET DIFF*
> ENV: {}
> {} -> {}
",
        env,
        last_balance_update.to_rfc3339(),
        utc_now.to_rfc3339()
    );

    for (asset, diff) in diff_vec.iter() {
        if *diff == 0.0 {
            continue;
        }
        msg.push_str(format!("{}: {}\n", asset, diff).as_str())
    }

    match sl_client
        .send_message(String::from("alert-virtual-taker-1"), msg)
        .await
    {
        Ok(()) => Ok(()),
        Err(e) => Err(anyhow::format_err!("{}", e)),
    }
}

async fn send_balances_msg(
    sl_client: &slackclient::client::Client,
    env: &str,
    balances: &HashMap<String, f64>,
) -> anyhow::Result<()> {
    let mut balances_vec: Vec<(String, f64)> =
        balances.iter().map(|(k, v)| (k.clone(), *v)).collect();

    balances_vec.sort_unstable_by(|a, b| a.0.cmp(&b.0));

    let mut msg = format!(
        "******
*BALANCES*
> ENV: {}
",
        env
    );

    for (asset, diff) in balances_vec.iter() {
        msg.push_str(format!("{}: {}\n", asset, diff).as_str());
    }

    match sl_client
        .send_message(String::from("alert-virtual-taker-1"), msg)
        .await
    {
        Ok(()) => Ok(()),
        Err(e) => Err(anyhow::format_err!("{}", e)),
    }
}
