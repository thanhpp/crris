mod slackclient;

use serde::Deserialize;
use std::{error::Error, fs};

#[derive(serde::Deserialize, Clone)]
pub struct Config {
    pub cex_dex_config: Vec<CexDexConfig>,
    pub slack_client_config: slackclient::client::SlackClientConfig,
}

#[derive(serde::Serialize, Deserialize, Clone)]
pub struct CexDexConfig {
    pub env: String,
    pub urls: Vec<URL>,
}

#[derive(serde::Serialize, Deserialize, Clone)]
pub struct URL {
    pub base_url: String,
    pub user: String,
    pub pass: String,
}

impl Config {
    pub fn from_yaml(path: String) -> Result<Config, Box<dyn Error>> {
        let contents = fs::read_to_string(path)?;
        let cfg: Config = serde_yaml::from_str(&contents)?;

        Ok(cfg)
    }
}
