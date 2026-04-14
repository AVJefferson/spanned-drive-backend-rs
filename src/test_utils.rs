#[cfg(test)]
pub mod timeout {
    use std::future::Future;
    use std::time::Duration;

    /// The standard timeout globally defined for all tests.
    pub const STANDARD_TIMEOUT: Duration = Duration::from_secs(5);

    /// Enforces the global standard timeout on a test future.
    pub async fn with_default<F: Future>(fut: F) -> F::Output {
        tokio::time::timeout(STANDARD_TIMEOUT, fut)
            .await
            .expect("Test execution exceeded the standard timeout!")
    }

    /// Provides a central and easily recognizable mechanism for overriding the test timeout.
    pub async fn with_override<F: Future>(duration: Duration, fut: F) -> F::Output {
        tokio::time::timeout(duration, fut)
            .await
            .expect("Test execution exceeded the overridden timeout!")
    }
}
