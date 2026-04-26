use anyhow::Result;
use async_trait::async_trait;
use std::{sync::Arc, time::Duration};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::external_systems::ExternalClients;

#[async_trait]
pub trait Task: Send + Sync {
    fn name(&self) -> String;
    async fn run(&self, clients: Arc<ExternalClients>) -> Result<()>;
}

pub enum Schedule {
    Cron(String),
    Interval(u64),
    Once(u64),
}

pub struct Scheduler {
    scheduler: JobScheduler,
    clients: Arc<ExternalClients>,
}

impl Scheduler {
    pub async fn new() -> Result<Self> {
        let scheduler = JobScheduler::new().await?;
        let clients = Arc::new(ExternalClients::new_from_env_variables().await);
        info!("Scheduler initialized");
        Ok(Self { scheduler, clients })
    }

    pub async fn add_task(&mut self, task: Box<dyn Task + Send + Sync>, schedule: Schedule) {
        let task_name = task.name();
        let task = Arc::new(task);
        let clients = self.clients.clone();

        let job_result = match schedule {
            Schedule::Cron(expr) => {
                let task = Arc::clone(&task);
                Job::new_async(expr.as_str(), move |_, _| {
                    let task = Arc::clone(&task);
                    let clients = clients.clone();
                    Box::pin(async move {
                        info!("Running cron task: {}", task.name());
                        if let Err(e) = task.run(clients).await {
                            error!("Task {} failed: {:?}", task.name(), e);
                        } else {
                            info!("Task {} completed", task.name());
                        }
                    })
                })
            }
            Schedule::Interval(secs) => {
                let task = Arc::clone(&task);
                Job::new_repeated_async(Duration::from_secs(secs), move |_, _| {
                    let task = Arc::clone(&task);
                    let clients = clients.clone();
                    Box::pin(async move {
                        info!("Running interval task: {}", task.name());
                        if let Err(e) = task.run(clients).await {
                            error!("Task {} failed: {:?}", task.name(), e);
                        } else {
                            info!("Task {} completed", task.name());
                        }
                    })
                })
            }
            Schedule::Once(delay) => {
                let task = Arc::clone(&task);
                Job::new_one_shot_async(Duration::from_secs(delay), move |_, _| {
                    let task = Arc::clone(&task);
                    let clients = clients.clone();
                    Box::pin(async move {
                        info!("Running one-shot task: {}", task.name());
                        if let Err(e) = task.run(clients).await {
                            error!("Task {} failed: {:?}", task.name(), e);
                        } else {
                            info!("Task {} completed", task.name());
                        }
                    })
                })
            }
        };

        match job_result {
            Ok(job) => {
                if let Err(e) = self.scheduler.add(job).await {
                    error!("Failed to add task {} to scheduler: {:?}", task_name, e);
                } else {
                    info!("Task {} added to scheduler", task_name);
                }
            }
            Err(e) => {
                error!("Failed to create job for task {}: {:?}", task_name, e);
            }
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting scheduler");
        self.scheduler.start().await?;
        Ok(())
    }
}
