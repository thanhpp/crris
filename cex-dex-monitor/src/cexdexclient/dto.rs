use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Response {
    result: ResponseResult,
    data: Vec<StateData>,
}

#[derive(Deserialize, Debug)]
pub struct ResponseResult {
    code: i64,
    message: String,
}

#[derive(Deserialize, Debug)]
pub struct StateData {
    pub state_id: String,
    pub cex: String,
    pub dex: String,
    pub dex_chain: String,
    pub dex_wallet: String,
    pub token: String,
    pub base_amount: f64,
    pub side: String,
    pub p1_price_diff: f64,
    pub p1_profitable_threshold: f64,
    pub p1_fillable_threshold: f64,
    pub p2_cancel_threshold: f64,
    pub is_done: bool,
    pub created_time: String,
    pub p2_total_gas: f64,
    pub slippage_percent: f64,
    pub p1_cex_orders: Option<Vec<CexOrderData>>,
    pub p2_cex_orders: Option<Vec<CexOrderData>>,
    // p2_dex_txs: String,
    // asset_change: String,
    // asset_change_with_fee: String,
}

#[derive(Deserialize, Debug)]
pub struct CexOrderData {
    pub id: String,
    pub status: String,
    pub base_symbol: String,
    pub quote_symbol: String,
    pub side: String,
    pub actual_price: f64,
    pub actual_price_with_fee: f64,
    pub base_amount: f64,
    pub filled_base_amount: f64,
    pub filled_quote_amount: f64,
    pub fee_asset: String,
    pub fee_amount: f64,
    pub filled_at: i64,
    pub created_time: i64,
}
