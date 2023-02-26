use std::{error::Error, fs};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub tpp_slack_bot_config: TPPSlackBotConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TPPSlackBotConfig {
    pub slack_ws_token: String,
    pub slack_api_token: String,
    pub kyber_dev_binance_read_api_key: String,
    pub kyber_dev_binance_read_secret_key: String,
}

impl TPPSlackBotConfig {
    pub fn from_yaml(path: &str) -> Result<TPPSlackBotConfig, Box<dyn Error>> {
        let file_content = fs::read_to_string(path)?;
        let app_config = serde_yaml::from_str::<AppConfig>(&file_content)?;

        Ok(app_config.tpp_slack_bot_config)
    }
}

#[cfg(test)]
mod tests {
    use super::TPPSlackBotConfig;

    #[test]
    fn test_write_config() {
        let cfg = TPPSlackBotConfig {
            slack_ws_token: String::from("slack_ws_token"),
            slack_api_token: String::from("slack_api_token"),
            kyber_dev_binance_read_api_key: String::from("kyber_dev_binance_read_api_key"),
            kyber_dev_binance_read_secret_key: String::from("kyber_dev_binance_read_secret_key"),
        };

        let data = serde_yaml::to_string(&cfg).unwrap();

        println!("{}", data);
    }
}
