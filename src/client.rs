use crate::{connection::Connection, AuthRequest, AuthResponse, NewTask};
use crate::{NewSolution, PoolMessage};
use snarkvm::console::network::MainnetV0;
// Client state
pub struct Connected;
pub struct Authed;

pub struct MainnetClient<S> {
    connection: Connection<MainnetV0>,
    state: S,
}

pub async fn connect(dest: &str) -> anyhow::Result<MainnetClient<Connected>> {
    let (ws, _) = tokio_tungstenite::connect_async(dest).await?;
    let connection = Connection::new(ws, true);
    Ok(MainnetClient {
        connection,
        state: Connected,
    })
}

impl MainnetClient<Connected> {
    pub async fn auth(mut self, request: AuthRequest) -> anyhow::Result<(MainnetClient<Authed>, AuthResponse<MainnetV0>)> {
        self.connection.send(PoolMessage::AuthRequest(request)).await?;
        let response = self.connection.next_auth_response().await?;
        let client = MainnetClient {
            connection: self.connection,
            state: Authed,
        };
        Ok((client, response))
    }
}

impl MainnetClient<Authed> {
    pub async fn next_task(&mut self) -> anyhow::Result<NewTask<MainnetV0>> {
        let task = self.connection.next_task().await?;
        Ok(task)
    }

    pub async fn send_solution(&self, solution: NewSolution<MainnetV0>) -> anyhow::Result<()> {
        self.connection.send(PoolMessage::NewSolution(solution)).await?;
        Ok(())
    }
}
