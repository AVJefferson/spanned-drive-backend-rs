use crate::logger::config::LoggerConfig;
use anyhow::Result;

use tracing_log::LogTracer;

#[derive(Debug, Clone)]
pub struct Logger {}

impl Logger {
    pub fn new(_channel_size: usize) -> Result<Self> {
        let config = LoggerConfig::from_env();

        if config.enable_log_tracing {
            let _ = LogTracer::init();
        }

        let _ = tracing_subscriber::fmt()
            .with_max_level(config.log_level)
            .try_init();

        Ok(Self {})
    }
}
