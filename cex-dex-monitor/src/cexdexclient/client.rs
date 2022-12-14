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
    pub fn new(base_url: String, user: String, pass: String) -> CexDexClient {
        CexDexClient {
            client: reqwest::Client::new(),
            base_url: base_url,
            user: user,
            pass: pass,
        }
    }

    pub async fn get_states(&self) -> Result<Response, Box<dyn Error + Send + Sync>> {
        let body = self
            .client
            .get(format!("{}/state", self.base_url))
            .query(&[("is_done", "true"), ("fill_state", "true"), ("size", "50")])
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
}
