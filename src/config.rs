use anyhow::Result;
use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct Config {
    pub endpoint: String,
    pub rootCAPath: String,
    pub certificatePath: String,
    pub privateKeyPath: String,
    pub port: i32,
    pub clientID: String,
    pub region: String,
    pub retryWaitTime: i32,
    pub retryAttempts: i32,
}

impl Config {
    pub fn try_from_path(path: &str) -> Result<Self> {
        let config_file = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&config_file)?;
        Ok(config)
    }
}
