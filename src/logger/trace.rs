use std::process;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TRACE_COUNTER: AtomicU32 = AtomicU32::new(0);

#[cfg(feature = "enable_log_tracing")]
pub fn generate() -> String {
    let pid = process::id();

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();

    let counter = TRACE_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    let counter = if counter > 999_999 {
        TRACE_COUNTER.store(1, Ordering::Relaxed);
        1
    } else {
        counter
    };

    format!("{:06}{:013}{:06}", pid, timestamp_ms, counter)
}

#[cfg(not(feature = "enable_log_tracing"))]
pub fn generate() -> String {
    "".to_string()
}
