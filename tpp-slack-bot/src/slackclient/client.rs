use std::{error::Error, sync::Arc};

use futures_util::{SinkExt, StreamExt}; // split websocket stream
use serde::{Deserialize, Serialize};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[allow(dead_code)]

pub struct SlackClient {
    ws_token: String,
    api_token: String,
    http_client: reqwest::Client,
}

impl SlackClient {
    pub fn new(ws_token: String, api_token: String) -> SlackClient {
        SlackClient {
            ws_token: ws_token,
            api_token: api_token,
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn send_message(&self, channel: String, text: String) -> Result<(), Box<dyn Error>> {
        let req = SlackSendMessageReq {
            channel: channel,
            text: text,
        };

        let req_serialized = serde_json::to_string(&req)?;

        let resp_text = self
            .http_client
            .post("https://slack.com/api/chat.postMessage")
            .header("Content-type", "application/json; charset=utf-8")
            .bearer_auth(&self.api_token)
            .body(req_serialized)
            .send()
            .await?
            .text()
            .await?;

        println!("send_message response: {}", resp_text);

        Ok(())
    }

    pub async fn get_ws_channel(&mut self) -> Receiver<String> {
        let (tx, rx): (Sender<String>, Receiver<String>) = tokio::sync::mpsc::channel(100);
        let client = self.http_client.clone();
        let ws_token = self.ws_token.clone();
        tokio::spawn(async move {
            loop {
                let ws = connect_ws(&client, &ws_token).await;
                tokio::io::stdout().write(b"connected ws \n").await.unwrap();
                let (mut write, read) = ws.split();
                let arc_write = Arc::new(Mutex::new(&mut write));

                read.for_each(|item| async {
                    let data = match item {
                        Ok(it) => it.into_data(),
                        Err(e) => {
                            tokio::io::stdout()
                                .write(format!("read data error: {}\n", e).as_bytes())
                                .await
                                .unwrap();
                            return;
                        }
                    };
                    let str_data = String::from_utf8(data).unwrap();
                    tokio::io::stdout()
                        .write(format!("WS RECEIVE DATA {}\n", &str_data).as_bytes())
                        .await
                        .unwrap();

                    // ack
                    if let Some(envelope_id) = get_envelope_id(&str_data).await {
                        let msg = SlackWSWithEnvelopeID {
                            envelope_id: envelope_id,
                        };
                        let mutex_write = Arc::clone(&arc_write);
                        let mut write_stream = mutex_write.lock().await;
                        (*write_stream)
                            .send(Message::text(serde_json::to_string(&msg).unwrap()))
                            .await
                            .unwrap();
                        tokio::io::stdout()
                            .write(format!("ACK envelope id {}\n", msg.envelope_id).as_bytes())
                            .await
                            .unwrap();
                    }

                    tx.clone().send(str_data).await.unwrap();
                })
                .await;

                tokio::io::stdout()
                    .write(b"ws loop ended \n")
                    .await
                    .unwrap();
            }
        });

        rx
    }
}

async fn connect_ws(
    http_client: &reqwest::Client,
    ws_token: &str,
) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
    let open_conn_resp_txt = http_client
        .post("https://slack.com/api/apps.connections.open")
        .header("Content-type", "application/x-www-form-urlencoded")
        .header("Authorization", format!("Bearer {}", ws_token))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let slack_open_conn_resp: SlackOpenConnResp = serde_json::from_str(&open_conn_resp_txt)
        .expect(format!("received data {}", open_conn_resp_txt).as_str());
    if !slack_open_conn_resp.ok {
        panic!("get slack open conn error {}", open_conn_resp_txt);
    }
    tokio::io::stdout()
        .write(b"GET WS URL OK \n")
        .await
        .unwrap();

    let ws_url = url::Url::parse(&slack_open_conn_resp.url.unwrap()).unwrap();
    let (ws_stream, _) = tokio_tungstenite::connect_async(ws_url).await.unwrap();
    tokio::io::stdout()
        .write(b"CONNECT WS OK \n")
        .await
        .unwrap();

    ws_stream
}

async fn get_envelope_id(data: &str) -> Option<String> {
    if let Ok(msg) = serde_json::from_str::<SlackWSWithEnvelopeID>(data) {
        return Some(msg.envelope_id);
    }

    None
}

#[derive(Serialize)]
pub struct SlackSendMessageReq {
    pub channel: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct SlackOpenConnResp {
    pub ok: bool,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SlackWSWithEnvelopeID {
    pub envelope_id: String,
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[tokio::test]
    async fn test_send_message() {
        let client = SlackClient::new(String::from(""), env::var("TPP_SLACK_API_TOKEN").unwrap());
        client
            .send_message(
                String::from("C04N96G28F9"),
                String::from("[TEST] hello, world"),
            )
            .await
            .unwrap();
    }

    // TPP_SLACK_WS_TOKEN
    #[tokio::test]
    async fn test_ws() {
        let mut client =
            SlackClient::new(env::var("TPP_SLACK_WS_TOKEN").unwrap(), String::from(""));
        let mut rx = client.get_ws_channel().await;

        for _ in 0..10 {
            let _ = rx.recv().await.unwrap();
        }
    }
}
