use anyhow::{Context, Result};
use binance_bot::BinanceClient;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

#[tokio::main]
async fn main() {
    let c = BinanceClient::new("".into(), "".into());

    c.get_open_orders().await.unwrap();
}

#[derive(Debug)]
pub struct SlackClient {
    token: String,
    http_client: reqwest::Client,
    ws_url: String,
}

impl SlackClient {
    pub async fn new(token: String) -> Result<SlackClient, Box<dyn Error>> {
        let mut c = SlackClient {
            token: token,
            http_client: reqwest::Client::new(),
            ws_url: String::from(""),
        };

        let ws_url = c.get_ws_url().await?;
        c.ws_url = ws_url;

        Ok(c)
    }

    async fn get_ws_url(&self) -> Result<String, Box<dyn Error>> {
        let resp_str = self
            .http_client
            .post("https://slack.com/api/apps.connections.open")
            .header("Content-type", "application/x-www-form-urlencoded")
            .header("Authorization", format!("Bearer {}", &self.token))
            .send()
            .await?
            .text()
            .await?;

        let resp: SlackOpenConnResp = match serde_json::from_str(&resp_str) {
            Ok(r) => r,
            Err(e) => {
                return Err(e.into());
            }
        };

        if !resp.ok {
            return Err(format!("response not ok, message {}", &resp.url).into());
        }

        Ok(resp.url.into())
    }

    async fn handle_ws(&self) -> Result<(), Box<dyn Error>> {
        let url =
            url::Url::parse(&self.ws_url).with_context(|| format!("parse url {}", self.ws_url))?;

        let (ws_stream, _) = tokio_tungstenite::connect_async(url)
            .await
            .with_context(|| "connect async")?;

        println!("ws connected");

        let (mut write, read) = ws_stream.split();
        let (tx, mut rx): (Sender<String>, Receiver<String>) = mpsc::channel(32);

        tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                let req = SlackSendMessageReq {
                    channel: String::from("C04N96G28F9"),
                    text: String::from("value"),
                };
                write
                    .send(serde_json::to_string(&req).unwrap().to_string().into())
                    .await
                    .unwrap();
            }
        });

        read.for_each(|msg| async {
            let mut data = match msg {
                Ok(m) => m.into_data(),
                Err(e) => {
                    let _ = tokio::io::stdout()
                        .write(format!("read data error {}", e).as_bytes())
                        .await;
                    return;
                }
            };

            data.push(b'\n');
            if let Err(e) = tokio::io::stdout().write_all(&data).await {
                println!("write data error {}", e);
                return;
            }

            tx.clone()
                .send(String::from_utf8_lossy(&data).to_string())
                .await
                .unwrap();
        })
        .await;

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct SlackOpenConnResp {
    pub ok: bool,
    pub url: String,
}

#[cfg(test)]
mod test {
    use std::env;

    use super::*;

    #[tokio::test]
    async fn test_slack_client() -> Result<(), Box<dyn Error>> {
        let c = SlackClient::new(env::var("TPP_SLACK_BOT").unwrap()).await?;
        println!("slack client created");
        c.handle_ws().await?;
        Ok(())
    }
}

#[derive(Deserialize)]
struct SlackCommandPayload {
    pub token: String,
    pub team_id: String,
    pub team_domain: String,
    pub channel_id: String,
    pub channel_name: String,
    pub user_id: String,
    pub user_name: String,
    pub command: String,
    pub text: String,
    pub response_url: String,
    pub trigger_id: String,
}

#[derive(Deserialize)]
struct SlackCommandMessage {
    pub payload: SlackCommandPayload,
    pub envelope_id: String,
}

#[derive(Serialize, Deserialize)]
struct SlackAckMessage {
    pub envelope_id: String,
}

// {
//     "payload": {
//       "token": "bHKJ2n9AW6Ju3MjciOHfbA1b",
//       "team_id": "T0SNL8S4S",
//       "team_domain": "maria",
//       "channel_id": "C15SASXJ6",
//       "channel_name": "general",
//       "user_id": "U0SNL8SV8",
//       "user_name": "rainer",
//       "command": "/randorilke",
//       "text": "",
//       "response_url": "https://rilke.slack.com/commands/T0SNL8S4S/37053613554/YMB2ZESDLNjNLqSFZ1quhNAh",
//       "trigger_id": "37053613634.26768298162.440952c06ef4de2653466a48fe495f93"
//     },
//     "envelope_id": "dbdd0ef3-1543-4f94-bfb4-133d0e6c1545",
//     "type": "slash_commands",
//     "accepts_response_payload": true
//   }

#[derive(Serialize)]
struct SlackSendMessageReq {
    pub channel: String,
    pub text: String,
}
