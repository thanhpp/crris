use hmac::{Hmac, Mac, NewMac};
use reqwest::header;
use std::{
    error::Error,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/*
https://tms-dev-blog.com/binance-api-crypto-transaction-with-rust-how-to/
*/

pub struct BinanceClient {
    client: reqwest::Client,
    api_key: String,
    secret_key: String,
}

impl BinanceClient {
    pub fn new(api_key: String, secret_key: String) -> Self {
        BinanceClient {
            client: reqwest::Client::new(),
            api_key: api_key,
            secret_key: secret_key,
        }
    }

    fn insert_api_key_header(
        &self,
        headers: &mut reqwest::header::HeaderMap,
    ) -> Result<(), Box<dyn Error>> {
        headers.insert(
            reqwest::header::HeaderName::from_static("x-mbx-apikey"),
            reqwest::header::HeaderValue::from_str(&self.api_key)?,
        );
        Ok(())
    }

    fn get_timestamp(&self, sys_time: SystemTime) -> Result<u128, Box<dyn Error>> {
        let since_epoch = sys_time.duration_since(UNIX_EPOCH)?;
        Ok(since_epoch.as_millis())
    }

    fn gen_signature(&self, request_params: &str) -> Result<String, Box<dyn Error>> {
        let mut signed_key =
            Hmac::<sha2::Sha256>::new_from_slice(&self.secret_key.as_bytes()).unwrap();
        signed_key.update(request_params.as_bytes());
        Ok(format!(
            "{}",
            hex::encode(signed_key.finalize().into_bytes())
        ))
    }

    pub async fn get_open_orders(&self) -> Result<(), Box<dyn Error>> {
        let mut headers = header::HeaderMap::new();
        self.insert_api_key_header(&mut headers)?;

        let timestamp = self.get_timestamp(
            SystemTime::now()
                .checked_sub(Duration::from_secs_f64(2.0))
                .unwrap(),
        )?;
        let params = format!("timestamp={}", timestamp.to_string());
        let signature = self.gen_signature(&params)?;

        let request_url = format!(
            "https://api.binance.com/api/v3/openOrders?{}&signature={}",
            &params, &signature,
        );
        println!("request_url: {}", &request_url);

        let result = self
            .client
            .get(request_url)
            .headers(headers)
            .send()
            .await?
            .text()
            .await?;

        println!("{}", result);

        Ok(())
    }
}
