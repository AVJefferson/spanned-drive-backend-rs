pub mod config;
pub mod trace;

#[cfg(feature = "enable_logging")]
pub mod logger;

#[cfg(not(feature = "enable_logging"))]
pub mod nologger;

#[cfg(feature = "enable_logging")]
pub use logger::Logger;

#[cfg(not(feature = "enable_logging"))]
pub use nologger::NoLogger as Logger;
