use std::fs;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug, Clone)]
pub struct MainConfig {
    pub telegram_token: String,
    pub google_secret_file: String,
    pub add_balance_config: AddBalanceConfig,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug, Clone)]
pub struct AddBalanceConfig {
    pub chat_id: String,
    pub sheet_id: String,
    pub write_range: String,
    pub balance_range: String,
}

impl MainConfig {
    pub fn new(path: &str) -> anyhow::Result<MainConfig> {
        let config_data = fs::read_to_string(path)?;
        let cfg: MainConfig = serde_yaml::from_str(&config_data)?;

        Ok(cfg)
    }
}
