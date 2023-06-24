use std::fs;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub priv_key_file: String,
    pub sheet_id: String,
    pub read_range: String,
}

impl Config {
    pub fn new(config_file: &str) -> anyhow::Result<Config> {
        let f_content = fs::read_to_string(config_file)?;
        let cfg = serde_json::from_str(&f_content)?;

        Ok(cfg)
    }
}
