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

    pub async fn get_states(&self) -> Result<Response, Box<dyn Error>> {
        let body = self
            .client
            .get(format!("{}/state", self.base_url))
            .query(&[("is_done", "true"), ("fill_state", "true"), ("size", "50")])
            .header(CONTENT_TYPE, "application/json")
            .basic_auth(&self.user, Some(&self.pass))
            .send()
            .await?
            .json::<Response>()
            .await?;

        Ok(body)
    }
}
