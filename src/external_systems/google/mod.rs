pub mod config;
mod appdata;
mod auth;
mod drive;
mod helpers;

use self::config::GoogleConfig;

pub struct GoogleClient {
    pub config: GoogleConfig,
    pub(self) http_client: reqwest::Client,
}

impl GoogleClient {
    pub async fn new(config: GoogleConfig) -> anyhow::Result<Self> {
        Ok(Self {
            config,
            http_client: reqwest::Client::new(),
        })
    }
}
