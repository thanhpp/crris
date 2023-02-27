use core::fmt;
use hmac::{Hmac, Mac, NewMac};
use reqwest::header::HeaderMap;
use serde::{self, Deserialize, Serialize};
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

    pub fn get_open_order_service(&self) -> GetOpenOrderService {
        GetOpenOrderService {
            ic: InternalClient {
                c: reqwest::Client::new(),
                api_key: self.api_key.clone(),
                secret_key: self.secret_key.clone(),
            },
        }
    }

    pub fn get_account_info_service(&self) -> GetAccountService {
        GetAccountService {
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
    pub async fn exec(&self) -> Result<Vec<BinanceOrder>, Box<dyn Error>> {
        let str_resp = self
            .ic
            .do_get_request_with_signature(
                String::from("https://api.binance.com/api/v3/openOrders"),
                &mut HashMap::new(),
            )
            .await?;

        if let Ok(open_orders) = serde_json::from_str::<Vec<BinanceOrder>>(&str_resp) {
            return Ok(open_orders);
        }

        Ok(Vec::new())
    }
}

#[derive(Deserialize, Debug)]
pub struct BinanceOrder {
    #[serde(alias = "symbol")]
    pub symbol: String,
    #[serde(alias = "orderId")]
    pub order_id: i64,
    #[serde(alias = "orderListId")]
    pub order_list_id: i64,
    #[serde(alias = "clientOrderId")]
    pub client_order_id: String,
    #[serde(alias = "price")]
    pub price: String,
    #[serde(alias = "origQty")]
    pub orig_qty: String,
    #[serde(alias = "executedQty")]
    pub executed_qty: String,
    #[serde(alias = "cummulativeQuoteQty")]
    pub cummulative_quote_qty: String,
    #[serde(alias = "status")]
    pub status: String,
    #[serde(alias = "timeInForce")]
    pub time_in_force: String,
    #[serde(alias = "type")]
    pub order_type: String,
    #[serde(alias = "side")]
    pub side: String,
    #[serde(alias = "stopPrice")]
    pub stop_price: String,
    #[serde(alias = "icebergQty")]
    pub iceberg_qty: String,
    #[serde(alias = "time")]
    pub time: i64,
    #[serde(alias = "updateTime")]
    pub update_time: i64,
    #[serde(alias = "isWorking")]
    pub is_working: bool,
    #[serde(alias = "workingTime")]
    pub working_time: i64,
    #[serde(alias = "origQuoteOrderQty")]
    pub orig_quote_order_qty: String,
    #[serde(alias = "selfTradePreventionMode")]
    pub self_trade_prevention_mode: String,
}

impl fmt::Display for BinanceOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "symbol: {}
order_id: {}
order_list_id: {}
client_order_id: {}
price: {}
orig_qty: {}
executed_qty: {}
cummulative_quote_qty: {}
status: {}
time_in_force: {}
order_type: {}
side: {}
stop_price: {}
iceberg_qty: {}
time: {}
update_time: {}
is_working: {}
working_time: {}
orig_quote_order_qty: {}
self_trade_prevention_mode: {}\n",
            self.symbol,
            self.order_id,
            self.order_list_id,
            self.client_order_id,
            self.price,
            self.orig_qty,
            self.executed_qty,
            self.cummulative_quote_qty,
            self.status,
            self.time_in_force,
            self.order_type,
            self.side,
            self.stop_price,
            self.iceberg_qty,
            self.time,
            self.update_time,
            self.is_working,
            self.working_time,
            self.orig_quote_order_qty,
            self.self_trade_prevention_mode,
        )
    }
}

pub struct GetAccountService {
    ic: InternalClient,
}

impl GetAccountService {
    pub async fn exec(&self) -> Result<AccountInfoResp, Box<dyn Error>> {
        let str_resp = self
            .ic
            .do_get_request_with_signature(
                String::from("https://api.binance.com/api/v3/account"),
                &mut HashMap::new(),
            )
            .await?;

        tokio::io::stdout()
            .write(&str_resp.as_bytes())
            .await
            .unwrap();

        if str_resp.len() == 0 {
            return Err("0 length string response".into());
        }

        match serde_json::from_str::<AccountInfoResp>(&str_resp) {
            Ok(resp) => {
                return Ok(resp);
            }
            Err(e) => {
                return Err(e.into());
            }
        };
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountInfoResp {
    #[serde(alias = "makerCommission")]
    pub maker_commission: i64,
    #[serde(alias = "takerCommission")]
    pub taker_commission: i64,
    #[serde(alias = "buyerCommission")]
    pub buyer_commission: i64,
    #[serde(alias = "sellerCommission")]
    pub seller_commission: i64,
    #[serde(alias = "commissionRates")]
    pub commission_rates: CommissionRates,
    #[serde(alias = "canTrade")]
    pub can_trade: bool,
    #[serde(alias = "canWithdraw")]
    pub can_withdraw: bool,
    #[serde(alias = "canDeposit")]
    pub can_deposit: bool,
    #[serde(alias = "brokered")]
    pub brokered: bool,
    #[serde(alias = "requireSelfTradePrevention")]
    pub require_self_trade_prevention: bool,
    #[serde(alias = "updateTime")]
    pub update_time: i64,
    #[serde(alias = "accountType")]
    pub account_type: String,
    #[serde(alias = "balances")]
    pub balances: Vec<Balance>,
    #[serde(alias = "permissions")]
    pub permissions: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommissionRates {
    pub maker: String,
    pub taker: String,
    pub buyer: String,
    pub seller: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: String,
    pub locked: String,
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
                    .checked_sub(Duration::from_secs_f64(3.0))
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
    ) -> Result<String, Box<dyn Error>> {
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
        let signature = self.gen_signature(&param_string)?;
        param_string.push_str(format!("&signature={}", signature).as_str());

        let send_url = format!("{}?{}", request_url, param_string);
        tokio::io::stdout()
            .write(format!("created binance send_url {}\n", &send_url).as_bytes())
            .await?;

        // add headers
        let mut headers = HeaderMap::new();
        self.insert_api_key_header(&mut headers);

        let resp = self
            .c
            .get(send_url)
            .headers(headers)
            .send()
            .await?
            .text()
            .await?;

        Ok(resp)
    }
}
