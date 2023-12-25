use std::sync::atomic::AtomicBool;

use crate::connection::read_pool_message_from_stream;
use crate::{connection::Connection, AuthRequest, AuthResponse, NewTask};
use crate::{NewSolution, PoolMessage};

use snarkvm::prelude::Network;
use tokio::sync::mpsc::Receiver;

pub struct Client<N: Network> {
    authed: AtomicBool,
    connection: Connection<N>,
}

impl<N: Network> Client<N> {
    fn is_authed(&self) -> bool {
        self.authed.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub async fn connect(dest: &str) -> anyhow::Result<Self> {
        let (ws, _) = tokio_tungstenite::connect_async(dest).await?;
        let connection = Connection::new(ws);
        Ok(Self {
            connection,
            authed: AtomicBool::new(false),
        })
    }

    pub async fn auth(&mut self, request: AuthRequest) -> anyhow::Result<AuthResponse<N>> {
        if self.is_authed() {
            return Err(anyhow::anyhow!("Already authed"));
        }
        self.connection.write_pool_message(PoolMessage::AuthRequest(request)).await?;
        let response = self
            .connection
            .read_pool_message()
            .await?
            .auth_response()
            .ok_or(anyhow::anyhow!("Unexpected message type"))?;
        self.authed.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(response)
    }

    pub async fn subscribe(&mut self) -> anyhow::Result<Receiver<anyhow::Result<NewTask<N>>>> {
        if !self.is_authed() {
            return Err(anyhow::anyhow!("Not authed"));
        }
        let mut incoming = self
            .connection
            .take_stream()
            .await?
            .ok_or(anyhow::anyhow!("Connection to server has broken or stream has been taken"))?;
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
            loop {
                // Only handle the PollMessage::NewTask from incoming stream,
                // break if any other message type is received
                match read_pool_message_from_stream::<N>(&mut incoming).await {
                    Ok(PoolMessage::NewTask(task)) => {
                        if let Err(_) = tx.send(Ok(task)).await {
                            break;
                        }
                    }
                    Ok(_) => {
                        if let Err(_) = tx.send(Err(anyhow::anyhow!("Unexpected message type"))).await {
                            break;
                        }
                    }
                    Err(err) => {
                        if let Err(_) = tx.send(Err(err)).await {
                            break;
                        }
                    }
                }
            }
        });
        Ok(rx)
    }

    pub async fn send_solution(&mut self, solution: NewSolution<N>) -> anyhow::Result<()> {
        if !self.is_authed() {
            return Err(anyhow::anyhow!("Not authed"));
        }
        self.connection.write_pool_message(PoolMessage::NewSolution(solution)).await?;
        Ok(())
    }
}
