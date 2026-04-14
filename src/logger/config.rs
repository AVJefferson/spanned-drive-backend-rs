use tracing::Level;

#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub enable_log_tracing: bool,
    pub log_level: Level,
}

impl LoggerConfig {
    pub fn from_env() -> Self {
        let enable_log_tracing = cfg!(feature = "enable_log_tracing");

        let log_level = if cfg!(feature = "enable_log_level_all") {
            Level::TRACE
        } else if cfg!(feature = "enable_log_level_verbose") {
            Level::DEBUG
        } else if cfg!(feature = "enable_log_level_info") {
            Level::INFO
        } else if cfg!(feature = "enable_log_level_warn") {
            Level::WARN
        } else if cfg!(feature = "enable_log_level_error") {
            Level::ERROR
        } else if cfg!(feature = "enable_log_level_critical") {
            Level::ERROR
        } else {
            Level::INFO // Default level
        };

        Self {
            enable_log_tracing,
            log_level,
        }
    }
}
