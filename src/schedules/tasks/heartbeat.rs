use crate::{external_systems::ExternalClients, schedules::scheduler::Task};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::task;

pub struct HeartbeatTask;

#[async_trait]
impl Task for HeartbeatTask {
    fn name(&self) -> String {
        "heartbeat".to_string()
    }

    async fn run(&self, _clients: Arc<ExternalClients>) -> Result<()> {
        task::spawn_blocking(move || {
            // Task code goes here
        })
        .await?;

        Ok(())
    }
}
