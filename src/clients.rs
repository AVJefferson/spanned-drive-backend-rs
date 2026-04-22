use crate::external_systems::google;
use std::{env, sync::Arc};

#[derive(Clone)]
pub struct Clients {
    pub google: Option<Arc<google::GoogleClient>>,
}

impl Clients {
    pub async fn new_from_env_variables() -> Self {
        Self {
            google: Self::init_google().await,
        }
    }

    async fn init_google() -> Option<Arc<google::GoogleClient>> {
        if env::var("ENABLE_EXTERNAL_SYSTEM_GOOGLE").ok()? == "true" {
            let config = google::config::GoogleConfig {
                client_id: env::var("GOOGLE_CLIENT_ID").ok()?,
                client_secret: env::var("GOOGLE_CLIENT_SECRET").ok()?,
            };

            match google::GoogleClient::new(config).await {
                Ok(client) => {
                    Some(Arc::new(client))
                }
                Err(e) => {
                    println!("Google Client enabled but failed to init: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    #[cfg(test)]
    pub fn empty() -> Self {
        Self { google: None }
    }
}
