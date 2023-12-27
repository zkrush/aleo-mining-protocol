use std::sync::Arc;

use crate::NewTask;
use snarkvm::prelude::Network;
use tokio::sync::{RwLock, Semaphore};

#[derive(Clone)]
pub struct PubSub<N: Network> {
    limit: Arc<tokio::sync::Semaphore>,
    task_pub: tokio::sync::broadcast::Sender<NewTask<N>>,
    current_task: Arc<RwLock<Option<NewTask<N>>>>,
}

impl<N: Network> PubSub<N> {
    pub fn new(limit: usize) -> Self {
        let limit = Arc::new(Semaphore::new(limit));
        let (task_pub, mut task_sub) = tokio::sync::broadcast::channel(1);
        let current_task = Arc::new(RwLock::new(None));
        {
            // Spawn a task to update the current task
            let current_task = current_task.clone();
            tokio::spawn(async move {
                loop {
                    let task = match task_sub.recv().await {
                        Ok(task) => task,
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    };
                    *current_task.write().await = Some(task);
                }
            });
        }

        Self {
            limit,
            task_pub,
            current_task,
        }
    }

    // Wrap the `broadcast::Receiver`, so that we can send first task to the new sub
    pub fn add_sub(&self) -> anyhow::Result<tokio::sync::mpsc::Receiver<NewTask<N>>> {
        // Ensure we dont have too many subs
        let _permit = self.limit.clone().try_acquire_owned()?;
        let mut task_sub = self.task_pub.subscribe();
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let current_task = self.current_task.clone();
        tokio::spawn(async move {
            // Move the permit into the task, so that it is released when the task is dropped
            let _permit = _permit;
            // 1. If we have task now, send it to the new subscriber
            if let Some(task) = current_task.read().await.clone() {
                // Ignore the error if the sub has gone
                if tx.send(task).await.is_err() {
                    return;
                }
            }
            // 2. Send the task to the sub when we `update_task`
            loop {
                match task_sub.recv().await {
                    Ok(task) => {
                        // Break the loop if the sub has gone
                        if (tx.send(task).await).is_err() {
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

    pub fn task_pub(&self) -> tokio::sync::broadcast::Sender<NewTask<N>> {
        self.task_pub.clone()
    }
}
