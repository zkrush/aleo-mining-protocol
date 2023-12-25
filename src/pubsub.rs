use std::sync::Arc;

use crate::NewTask;
use snarkvm::prelude::Network;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct PubSub<N: Network> {
    limit: Arc<tokio::sync::Semaphore>,
    task_pub: tokio::sync::broadcast::Sender<Arc<NewTask<N>>>,
    current_task: Option<Arc<NewTask<N>>>,
}

impl<N: Network> PubSub<N> {
    pub fn new(limit: usize, task_pub: tokio::sync::broadcast::Sender<Arc<NewTask<N>>>) -> Self {
        let limit = Arc::new(Semaphore::new(limit as usize));
        let current_task = None;
        Self {
            limit,
            task_pub,
            current_task,
        }
    }

    pub fn update_task(&mut self, task: NewTask<N>) {
        let task = Arc::new(task);
        self.current_task = Some(task.clone());

        // Only fail if there are no subs, so we just ignore the error
        let _ = self.task_pub.send(task.clone());
    }

    pub fn add_sub(&self) -> anyhow::Result<tokio::sync::mpsc::Receiver<Arc<NewTask<N>>>> {
        // Ensure we dont have too many subs
        let _permit = self.limit.clone().try_acquire_owned()?;
        let mut task_sub = self.task_pub.subscribe();
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let mut current_task = self.current_task.clone();
        tokio::spawn(async move {
            // Move the permit into the task, so that it is released when the task is dropped
            let _permit = _permit;
            // 1. If we have task now, send it to the new subscriber
            if let Some(task) = current_task.take() {
                // Ignore the error if the sub has gone
                if let Err(_) = tx.send(task).await {
                    return;
                }
            }
            // 2. Send the task to the sub when we `update_task`
            loop {
                match task_sub.recv().await {
                    Ok(task) => {
                        // Break the loop if the sub has gone
                        if let Err(_) = tx.send(task).await {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                    // Continue so that we will recv the latest task
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    }
                }
            }
        });
        Ok(rx)
    }
}
