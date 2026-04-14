use crate::logger::config::LoggerConfig;
use anyhow::Result;

use tracing_log::LogTracer;

#[derive(Debug, Clone)]
pub struct Logger {}

impl Logger {
    pub fn new(_channel_size: usize) -> Result<Self> {
        let config = LoggerConfig::from_env();

        if config.enable_log_tracing {
            LogTracer::init().expect("Failed to set logger");
        }

        let subscriber = tracing_subscriber::fmt::Subscriber::builder()
            .with_max_level(config.log_level)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global default subscriber");

        Ok(Self {})
    }
}
