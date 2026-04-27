use crate::logger;

use std::sync::LazyLock;

pub const DEFAULT_SERVER_PORT: &'static str = "3000";
pub const DEFAULT_SERVER_HOST: &'static str = "127.0.0.1";
pub const LOG_CHANNEL_SIZE: usize = 1000;

pub const SDRIVE_FOLDER_NAME: &'static str = ".sdrive";

pub static APP_TRACE_ID: LazyLock<String> = LazyLock::new(|| logger::trace::generate());
