use anyhow::Result;

#[derive(Debug, Clone)]
pub struct NoLogger {}

impl NoLogger {
    pub fn new(channel_size: usize) -> Result<Self> {
        Ok(Self {})
    }
}
