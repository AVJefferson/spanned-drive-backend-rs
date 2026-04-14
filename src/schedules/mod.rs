mod scheduler;
mod tasks;

use scheduler::Schedule::Interval;

use tasks::heartbeat;

use tracing::info;

pub async fn init_scheduler() {
    info!("Initializing scheduler");
    let mut scheduler = scheduler::Scheduler::new().await.unwrap();

    scheduler
        .add_task(Box::new(heartbeat::HeartbeatTask), Interval(10))
        .await;

    // Start scheduler in the background
    tokio::spawn(async move {
        if let Err(e) = scheduler.start().await {
            tracing::error!("Scheduler failed: {:?}", e);
        }
    });
}
