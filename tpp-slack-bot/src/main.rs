mod binanceclient;
mod slackclient;
use binanceclient::client::{BinanceClient, BinanceOrder};
use config::TPPSlackBotConfig;
use serde::{self, Deserialize};
use serde_json;
use slackclient::client::{SlackClient, SlackSendMessageReq};
use tokio::{self, io::AsyncWriteExt};
mod config;

#[tokio::main]
async fn main() {
    let cfg = TPPSlackBotConfig::from_yaml("secret.yaml").expect("secret.yaml not found");

    let mut s_client = SlackClient::new(cfg.slack_ws_token, cfg.slack_api_token);

    let b_client = BinanceClient::new(
        cfg.kyber_dev_binance_read_api_key,
        cfg.kyber_dev_binance_read_secret_key,
    );

    let mut rx = s_client.get_ws_channel().await;

    loop {
        let data = rx.recv().await.unwrap();
        let msg_type = get_slack_ws_msg_type(&data);
        if msg_type.ne(&Some(String::from("slash_commands"))) {
            continue;
        }

        let slash_command_msg = serde_json::from_str::<SlackWSSlashCommandMsg>(&data).unwrap();
        tokio::io::stdout()
            .write(format!("SLACK SLASH COMMAND\n {:?}\n", slash_command_msg).as_bytes())
            .await
            .unwrap();

        match slash_command_msg.payload.command.as_str() {
            "/openorders" => {
                let binance_orders = b_client.get_open_order_service().exec().await.unwrap();

                let post_response = reqwest::Client::new()
                    .post(url::Url::parse(&slash_command_msg.payload.response_url).unwrap())
                    .body(
                        serde_json::to_string(&SlackSendMessageReq {
                            channel: slash_command_msg.payload.channel_id,
                            text: stringtify_binance_orders(&binance_orders),
                        })
                        .unwrap(),
                    )
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap();
                tokio::io::stdout()
                    .write(format!("POST RESPONSE {}\n", post_response).as_bytes())
                    .await
                    .unwrap();
            }
            _ => {}
        }
    }
}

fn get_slack_ws_msg_type(data: &str) -> Option<String> {
    if let Ok(msg) = serde_json::from_str::<SlackWSMsgWithType>(data) {
        return Some(msg.msg_type);
    }
    None
}

fn stringtify_binance_orders(v: &Vec<BinanceOrder>) -> String {
    if v.len() == 0 {
        return String::from("no order found");
    }

    let mut str_resp = String::from("");
    for ord in v {
        str_resp = str_resp + format!("{}\n", ord).as_str();
    }

    str_resp
}

#[derive(Deserialize, Debug)]
struct SlackWSMsgWithType {
    #[serde(alias = "type")]
    msg_type: String,
}

#[derive(Deserialize, Debug)]
struct SlackWSSlashCommandMsg {
    payload: SlackWSSlashCommandPayload,
}

#[derive(Deserialize, Debug)]
struct SlackWSSlashCommandPayload {
    token: String,
    team_id: String,
    team_domain: String,
    channel_id: String,
    channel_name: String,
    user_id: String,
    user_name: String,
    command: String,
    text: String,
    api_app_id: String,
    is_enterprise_install: String,
    response_url: String,
    trigger_id: String,
}
