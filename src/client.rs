use std::sync::atomic::AtomicBool;

use futures_util::stream::BoxStream;
use snarkvm::prelude::Network;

use crate::{connection::Connection, AuthResponse, AuthRequest, NewTask};

pub struct Client<N: Network> {
    authed: AtomicBool,
    connection: Connection<N>,
}

impl<N: Network> Client<N> {
    fn is_authed(&self) -> bool {
        self.authed.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub async fn connect(dest: &str) -> anyhow::Result<Self> {
        let (ws, _ ) = tokio_tungstenite::connect_async(dest).await?;
        let connection = Connection::new(ws);
        Ok(Self { connection, authed: AtomicBool::new(false)})
    }

    pub async fn auth(&mut self, request: AuthRequest) -> anyhow::Result<AuthResponse<N>> {
        if self.is_authed() {
            return Err(anyhow::anyhow!("Already authed"));
        }
        todo!()
    }

    pub async fn subscribe(&mut self) -> anyhow::Result<BoxStream<NewTask<N>>> {
        if !self.is_authed() {
            return Err(anyhow::anyhow!("Not authed"));
        }
        let incoming = self.connection.take_stream().await?;
        todo!()
    }

    pub async fn send_solution(&mut self) -> anyhow::Result<()> {
        if !self.is_authed() {
            return Err(anyhow::anyhow!("Not authed"));
        }
        todo!()
    }
}
