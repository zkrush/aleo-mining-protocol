use crate::{connection::Connection, AuthRequest, AuthResponse, NewTask};
use crate::{NewSolution, PoolMessage};
use std::sync::atomic::AtomicBool;

use snarkvm::prelude::Network;

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
        let connection = Connection::new(ws, true);
        Ok(Self {
            connection,
            authed: AtomicBool::new(false),
        })
    }

    pub async fn auth(&mut self, request: AuthRequest) -> anyhow::Result<AuthResponse<N>> {
        if self.is_authed() {
            return Err(anyhow::anyhow!("Already authed"));
        }
        self.connection.send(PoolMessage::AuthRequest(request)).await?;
        let response = self.connection.next_auth_response().await?;
        self.authed.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(response)
    }

    pub async fn next_task(&mut self) -> anyhow::Result<NewTask<N>> {
        if !self.is_authed() {
            return Err(anyhow::anyhow!("Not authed"));
        }
        let task = self.connection.next_task().await?;
        Ok(task)
    }

    pub async fn send_solution(&self, solution: NewSolution<N>) -> anyhow::Result<()> {
        if !self.is_authed() {
            return Err(anyhow::anyhow!("Not authed"));
        }
        self.connection.send(PoolMessage::NewSolution(solution)).await?;
        Ok(())
    }
}
