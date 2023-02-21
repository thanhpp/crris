use std::env;

mod slackclient;

use serde::{self, Deserialize};
use serde_json;
use slackclient::client::{SlackClient, SlackSendMessageReq};
use tokio::{self, io::AsyncWriteExt};

#[tokio::main]
async fn main() {
    let mut s_client = SlackClient::new(
        env::var("TPP_SLACK_WS_TOKEN").unwrap(),
        env::var("TPP_SLACK_API_TOKEN").unwrap(),
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

        let post_response = reqwest::Client::new()
            .post(url::Url::parse(&slash_command_msg.payload.response_url).unwrap())
            .body(
                serde_json::to_string(&SlackSendMessageReq {
                    channel: slash_command_msg.payload.channel_id,
                    text: String::from("response"),
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
}

fn get_slack_ws_msg_type(data: &str) -> Option<String> {
    if let Ok(msg) = serde_json::from_str::<SlackWSMsgWithType>(data) {
        return Some(msg.msg_type);
    }
    None
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
