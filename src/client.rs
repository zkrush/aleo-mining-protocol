use crate::{connection::Connection, AuthRequest, AuthResponse, NewTask};
use crate::{NewSolution, PoolMessage};
use snarkvm::prelude::Network;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

// Client state
pub struct Connected;
pub struct Authed;

pub struct PoolClient<S, N: Network> {
    connection: Connection<N>,
    _state: S,
}

pub async fn connect<N: Network>(dest: &str) -> anyhow::Result<PoolClient<Connected, N>> {
    let (ws, _) = tokio_tungstenite::connect_async_with_config(dest, Some(WebSocketConfig::default()), false).await?;
    let connection = Connection::new(ws);
    Ok(PoolClient {
        connection,
        _state: Connected,
    })
}

impl<N: Network> PoolClient<Connected, N> {
    pub async fn auth(mut self, request: AuthRequest) -> anyhow::Result<(PoolClient<Authed, N>, AuthResponse<N>)> {
        self.connection.send(PoolMessage::AuthRequest(request)).await?;
        let response = self.connection.next_auth_response().await?;
        let client = PoolClient {
            connection: self.connection,
            _state: Authed,
        };
        Ok((client, response))
    }
}

impl<N: Network> PoolClient<Authed, N> {
    pub async fn next_task(&mut self) -> anyhow::Result<NewTask<N>> {
        let task = self.connection.next_task().await?;
        Ok(task)
    }

    pub async fn send_solution(&self, solution: NewSolution<N>) -> anyhow::Result<()> {
        self.connection.send(PoolMessage::NewSolution(solution)).await?;
        Ok(())
    }
}
