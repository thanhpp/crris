#![allow(dead_code)]

use std::{
    collections::HashMap,
    error::{self, Error},
    fmt,
};

use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    webhooks: HashMap<String, String>,
}

impl Client {
    pub fn new() -> Client {
        Client {
            client: reqwest::Client::new(),
            webhooks: HashMap::new(),
        }
    }

    pub fn add_webhook(&mut self, channel: String, web_hook: String) {
        self.webhooks.insert(channel, web_hook);
    }

    pub async fn send_message(&self, channel: String, msg: String) -> Result<(), Box<dyn Error>> {
        // get webhook urlBox
        let webhook = self.webhooks.get(&channel);
        if webhook.is_none() {
            return Err(ChannelNotFound.into());
        }
        let webhook = webhook.unwrap();

        // craft json message
        let req = SendRequest { text: msg };
        let req_text = serde_json::to_string(&req)?;

        // send message
        let resp = self
            .client
            .post(webhook)
            .header(CONTENT_TYPE, "application/json")
            .body(req_text)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Result::Err(ResponseNotSuccess::new(resp.status().as_u16()).into());
        }

        Ok(())
    }
}

// config

#[derive(Deserialize, Clone)]
pub struct SlackClientConfig {
    pub webhooks: Option<Vec<WebhookConfig>>,
}

#[derive(Deserialize, Clone)]
pub struct WebhookConfig {
    pub channel: String,
    pub webhook: String,
}

// dto

#[derive(Serialize)]
struct SendRequest {
    text: String,
}

// errors

#[derive(Debug, Clone)]
pub struct ChannelNotFound;

impl fmt::Display for ChannelNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "channel not found")
    }
}

impl error::Error for ChannelNotFound {}

#[derive(Debug, Clone)]
pub struct ResponseNotSuccess {
    code: u16,
}

impl ResponseNotSuccess {
    pub fn new(code: u16) -> ResponseNotSuccess {
        ResponseNotSuccess { code }
    }
}

impl fmt::Display for ResponseNotSuccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "response not success, code {}", self.code)
    }
}

impl error::Error for ResponseNotSuccess {}
