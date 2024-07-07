#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub mod connection;
use serde::{Deserialize, Serialize};
use snarkvm::ledger::puzzle::Solution;
use snarkvm::prelude::{Address, Network};
use tokio_tungstenite::tungstenite::Message;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(bound(serialize = "N: Serialize", deserialize = "N: for<'a> Deserialize<'a>"))]
pub enum PoolMessage<N: Network> {
    #[serde(rename = "auth_request")]
    AuthRequest(AuthRequest),
    #[serde(rename = "auth_response")]
    AuthResponse(AuthResponse<N>),
    #[serde(rename = "task")]
    NewTask(NewTask<N>),
    #[serde(rename = "solution")]
    NewSolution(NewSolution<N>),
}

impl<N: Network> PoolMessage<N> {
    pub fn auth_request(self) -> Option<AuthRequest> {
        match self {
            Self::AuthRequest(request) => Some(request),
            _ => None,
        }
    }

    pub fn auth_response(self) -> Option<AuthResponse<N>> {
        match self {
            Self::AuthResponse(response) => Some(response),
            _ => None,
        }
    }

    pub fn new_task(self) -> Option<NewTask<N>> {
        match self {
            Self::NewTask(new_task) => Some(new_task),
            _ => None,
        }
    }

    pub fn new_solution(self) -> Option<NewSolution<N>> {
        match self {
            Self::NewSolution(new_solution) => Some(new_solution),
            _ => None,
        }
    }
}

impl<N: Network> TryFrom<Message> for PoolMessage<N> {
    type Error = anyhow::Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        let message = match message {
            Message::Text(text) => text,
            _ => return Err(anyhow::anyhow!("Unexpected message type")),
        };
        serde_json::from_str(&message).map_err(anyhow::Error::msg)
    }
}

impl<N: Network> TryFrom<PoolMessage<N>> for Message {
    type Error = anyhow::Error;

    fn try_from(message: PoolMessage<N>) -> Result<Self, Self::Error> {
        let message = serde_json::to_string(&message).map_err(anyhow::Error::msg)?;
        Ok(Message::Text(message))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthRequest {
    // miner account or aleo address
    pub username: String,
    pub metadata: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound(serialize = "N: Serialize", deserialize = "N: for<'a> Deserialize<'a>"))]
pub struct AuthResponse<N: Network> {
    // Indicate the `AuthRequest` is success or not.
    pub result: bool,
    // Pool address.
    pub address: Address<N>,
    // Extra message
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "N: Serialize", deserialize = "N: for<'a> Deserialize<'a>"))]
pub struct NewTask<N: Network> {
    pub epoch_number: u32,
    pub epoch_hash: <N>::BlockHash,
    pub address: Address<N>,
    pub difficulty: u64,

    pub codes: Vec<u8>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "N: Serialize", deserialize = "N: for<'a> Deserialize<'a>"))]
pub struct NewSolution<N: Network> {
    pub epoch_number: u32,
    pub solution: Solution<N>,
}
