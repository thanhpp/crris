#![allow(dead_code)]

use super::dto::*;
use reqwest::header::CONTENT_TYPE;
use std::error::Error;

pub struct CexDexClient {
    client: reqwest::Client,
    base_url: String,
    user: String,
    pass: String,
}

impl CexDexClient {
    // https://github.com/hyperium/hyper/issues/2136
    pub fn new(base_url: String, user: String, pass: String) -> CexDexClient {
        CexDexClient {
            client: reqwest::Client::builder()
                .pool_max_idle_per_host(0)
                .pool_idle_timeout(None)
                .build()
                .unwrap(),
            base_url,
            user,
            pass,
        }
    }

    pub fn base_url(&self) -> String {
        self.base_url.clone()
    }

    pub async fn get_filled_done_states(&self) -> Result<Response, Box<dyn Error + Send + Sync>> {
        let body = self
            .client
            .get(format!("{}/state", self.base_url))
            .query(&[("is_done", "true"), ("fill_state", "true"), ("size", "20")])
            .header(CONTENT_TYPE, "application/json")
            .basic_auth(&self.user, Some(&self.pass))
            .send()
            .await?
            .text()
            .await?;

        let resp: Response = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                println!("serde_json error {}, body {}", e, &body);
                return Err(e.into());
            }
        };

        Ok(resp)
    }

    pub async fn get_running_states(&self) -> Result<Response, Box<dyn Error + Send + Sync>> {
        let body = self
            .client
            .get(format!("{}/state", self.base_url))
            .header(CONTENT_TYPE, "application/json")
            .basic_auth(&self.user, Some(&self.pass))
            .send()
            .await?
            .text()
            .await?;

        let resp = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                println!("serde_json error {}, body {}", e, &body);
                return Err(e.into());
            }
        };

        Ok(resp)
    }

    pub async fn get_cex_balanace(&self) -> anyhow::Result<GetCEXBalanceResponse> {
        let body = self
            .client
            .get(format!("{}/cex/binance/balances", self.base_url))
            .header(CONTENT_TYPE, "application/json")
            .basic_auth(&self.user, Some(&self.pass))
            .send()
            .await?
            .text()
            .await?;

        let resp = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                println!("serde_json error {}, body {}", e, &body);
                return Err(e.into());
            }
        };

        Ok(resp)
    }

    pub async fn get_dex_balanace(&self) -> anyhow::Result<GetDEXBalanceResponse> {
        let body = self
            .client
            .get(format!("{}/dex/polygon/balances", self.base_url))
            .header(CONTENT_TYPE, "application/json")
            .basic_auth(&self.user, Some(&self.pass))
            .send()
            .await?
            .text()
            .await?;

        let resp = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                println!(
                    "serde_json error [{}], body [{}], url [{}]",
                    e, &body, self.base_url
                );
                return Err(e.into());
            }
        };

        Ok(resp)
    }
}
