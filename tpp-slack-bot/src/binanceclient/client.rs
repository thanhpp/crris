use hmac::{Hmac, Mac, NewMac};
use reqwest::header::HeaderMap;
use std::{
    collections::HashMap,
    error::Error,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::io::AsyncWriteExt;

pub struct BinanceClient {
    api_key: String,
    secret_key: String,
}

impl BinanceClient {
    pub fn new(api_key: String, secret_key: String) -> BinanceClient {
        BinanceClient {
            api_key: api_key,
            secret_key: secret_key,
        }
    }

    pub fn GetOpenOrderService(&self) -> GetOpenOrderService {
        GetOpenOrderService {
            ic: InternalClient {
                c: reqwest::Client::new(),
                api_key: self.api_key.clone(),
                secret_key: self.secret_key.clone(),
            },
        }
    }
}

pub struct GetOpenOrderService {
    ic: InternalClient,
}

impl GetOpenOrderService {
    pub async fn Do(&self) -> String {
        self.ic
            .do_get_request_with_signature(
                String::from("https://api.binance.com/api/v3/openOrders"),
                &mut HashMap::new(),
            )
            .await
    }
}

struct InternalClient {
    c: reqwest::Client,
    api_key: String,
    secret_key: String,
}

impl InternalClient {
    fn get_timestamp(&self, sys_time: SystemTime) -> Result<u128, Box<dyn Error>> {
        let since_epoch = sys_time.duration_since(UNIX_EPOCH)?;
        Ok(since_epoch.as_millis())
    }

    fn gen_timestamp_param(&self) -> String {
        let timestamp = self
            .get_timestamp(
                SystemTime::now()
                    .checked_sub(Duration::from_secs_f64(2.0))
                    .unwrap(),
            )
            .unwrap();
        timestamp.to_string()
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

    fn insert_api_key_header(&self, headers: &mut reqwest::header::HeaderMap) {
        headers.insert(
            reqwest::header::HeaderName::from_static("x-mbx-apikey"),
            reqwest::header::HeaderValue::from_str(&self.api_key).unwrap(),
        );
    }

    async fn do_get_request_with_signature(
        &self,
        request_url: String,
        params: &mut HashMap<String, String>,
    ) -> String {
        params.insert(String::from("timestamp"), self.gen_timestamp_param());

        // build param string
        let mut param_string = String::from("");
        let mut is_first = true;
        for (k, v) in params {
            if is_first {
                param_string.push_str(format!("{}={}", k, v).as_str());
                is_first = false;
                continue;
            }
            param_string.push_str(format!("&{}={}", k, v).as_str());
        }

        // gen signature
        let signature = self.gen_signature(&param_string).unwrap();
        param_string.push_str(format!("&signature={}", signature).as_str());

        let send_url = format!("{}?{}", request_url, param_string);
        tokio::io::stdout()
            .write(format!("created binance send_url {}\n", &send_url).as_bytes())
            .await
            .unwrap();

        // add headers
        let mut headers = HeaderMap::new();
        self.insert_api_key_header(&mut headers);

        self.c
            .get(send_url)
            .headers(headers)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    }
}
