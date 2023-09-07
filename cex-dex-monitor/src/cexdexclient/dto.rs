use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub result: ResponseResult,
    pub data: Vec<StateData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseResult {
    pub code: i64,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
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
    pub slippage: f64,
    pub p1_cex_orders: Option<Vec<CexOrderData>>,
    pub p2_cex_orders: Option<Vec<CexOrderData>>,
    pub p2_dex_txs: Option<Vec<DexTxData>>,
    pub asset_change: Option<HashMap<String, f64>>,
    pub asset_change_with_fee: Option<HashMap<String, f64>>,
}

impl StateData {
    fn sum_filled(&self, v: &[CexOrderData], f: fn(&CexOrderData) -> f64) -> f64 {
        v.iter().map(f).sum()
    }

    pub fn get_cex_price(&self, o: &Option<Vec<CexOrderData>>) -> f64 {
        match o {
            None => 0.0,
            Some(v) => {
                let base_filled = self.sum_filled(v, |x| x.filled_base_amount);
                let quote_filled = self.sum_filled(v, |x| x.filled_quote_amount);
                if base_filled == 0.0 {
                    return 0.0;
                }
                quote_filled / base_filled
            }
        }
    }

    // part 1 cex
    pub fn count_p1_filled_orders(&self) -> usize {
        match &self.p1_cex_orders {
            None => 0,
            Some(v) => v.len(),
        }
    }

    pub fn p1_sum_base_filled(&self) -> f64 {
        match &self.p1_cex_orders {
            None => 0.0,
            Some(v) => self.sum_filled(v, |x| x.filled_base_amount),
        }
    }

    pub fn p1_sum_quote_filled(&self) -> f64 {
        match &self.p1_cex_orders {
            None => 0.0,
            Some(v) => self.sum_filled(v, |x| x.filled_quote_amount),
        }
    }

    // part 2 cex
    pub fn count_p2_filled_orders(&self) -> usize {
        match &self.p2_cex_orders {
            None => 0,
            Some(v) => v.len(),
        }
    }

    pub fn p2_sum_base_filled(&self) -> f64 {
        match &self.p2_cex_orders {
            None => 0.0,
            Some(v) => self.sum_filled(v, |x| x.filled_base_amount),
        }
    }

    pub fn p2_sum_quote_filled(&self) -> f64 {
        match &self.p2_cex_orders {
            None => 0.0,
            Some(v) => self.sum_filled(v, |x| x.filled_quote_amount),
        }
    }

    // part 2 dex
    pub fn p2_count_created_txs(&self) -> usize {
        match &self.p2_dex_txs {
            None => 0,
            Some(v) => v.len(),
        }
    }

    pub fn p2_sum_token_filled(&self, token: &String) -> f64 {
        match &self.p2_dex_txs {
            None => 0.0,
            Some(v) => v
                .iter()
                .map(|x| {
                    if !x.status.eq("EXECUTED") {
                        return 0.0;
                    }
                    if x.token_in.eq(token) {
                        x.amount_in
                    } else {
                        x.actual_amount_out
                    }
                })
                .sum(),
        }
    }

    pub fn p2_summary_txs(&self) -> String {
        let mut summary = String::from("");
        if let Some(txs) = &self.p2_dex_txs {
            for tx in txs {
                summary.push_str(&format!("{}: {}\n", tx.tx_hash.clone(), tx.status.clone()))
            }
        }

        summary
    }

    pub fn asset_changes(&self) -> String {
        let mut asset_changes = String::from("");
        if let Some(m) = &self.asset_change_with_fee {
            for (k, v) in m.iter() {
                asset_changes.push_str(&format!("- {}: {}\n", k, v));
            }
        }
        asset_changes
    }
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct DexTxData {
    pub nonce: i64,
    pub tx_hash: String,
    pub status: String,
    pub to_wallet: String,
    pub router_address: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: f64,
    pub estimated_amount_out: f64,
    pub actual_amount_out: f64,
    pub estimated_price: f64,
    pub estimated_price_with_fee: f64,
    pub actual_price: f64,
    pub actual_price_with_fee: f64,
    pub gas_price: f64,
    pub gas_used: u64,
    pub max_tip: f64,
    pub estimated_at: i64,
    pub broadcasted_at: i64,
    pub mined_at: i64,
    pub mined_block: u64,
    pub native_token_price_in_quote: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetCEXBalanceResponse {
    pub result: ResponseResult,
    pub data: CEXBalanceData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CEXBalanceData {
    pub id: String,
    pub balances: HashMap<String, CEXBalance>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CEXBalance {
    pub free: f64,
    pub locked: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDEXBalanceResponse {
    pub result: ResponseResult,
    pub data: DEXBalanceData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DEXBalanceData {
    pub id: String,
    pub balances: HashMap<String, f64>,
    pub contract_balances: HashMap<String, f64>,
}
