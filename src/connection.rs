use crate::{AuthRequest, AuthResponse, NewSolution, NewTask, PoolMessage};

use futures_util::{
    stream::{SplitSink as FutureSplitSink, SplitStream as FutureSplitStream},
    StreamExt,
};
use futures_util::SinkExt;
use snarkvm::prelude::Network;
use std::marker::PhantomData;

use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream as TokioWebSocketStream};

type WebSocketStream = TokioWebSocketStream<MaybeTlsStream<TcpStream>>;
pub type SplitStream = FutureSplitStream<WebSocketStream>;
pub type SplitSink = FutureSplitSink<WebSocketStream, Message>;

pub struct Connection<N: Network> {
    incoming_rx: tokio::sync::mpsc::Receiver<PoolMessage<N>>,
    outgoing_tx: tokio::sync::mpsc::Sender<PoolMessage<N>>,
    _p: PhantomData<N>,
}

impl<N: Network> Connection<N> {
    pub fn new(ws: WebSocketStream, enable_hb: bool) -> Self {
        let (outgoing_tx, outgoing_rx) = tokio::sync::mpsc::channel(1);
        let (incoming_tx, incoming_rx) = tokio::sync::mpsc::channel(1);
        tokio::spawn(Self::handle_inner(ws, outgoing_rx, incoming_tx, enable_hb));
        Self {
            incoming_rx,
            outgoing_tx,
            _p: PhantomData,
        }
    }

    pub async fn handle_inner(
        mut ws: WebSocketStream,
        mut outgoing_rx: tokio::sync::mpsc::Receiver<PoolMessage<N>>,
        incoming_tx: tokio::sync::mpsc::Sender<PoolMessage<N>>,
        enable_hb: bool
    ) -> anyhow::Result<()> {
        let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(15));
        // 1.Receive messages from the outgoing_rx and send them though the websocket
        // 2.When receive from the websocket, send them through the incoming_tx
        // 3. Send ping to the websocket every 15 seconds
        loop {
            tokio::select! {
                message = outgoing_rx.recv() => {
                    let message: PoolMessage<N> = message.ok_or_else(|| anyhow::anyhow!("client dropped"))?;
                    ws.send(message.into()).await?;
                }
                result = ws.next() => {
                    let message: Message = result.ok_or_else(|| anyhow::anyhow!("connection closed"))??;
                    match message {
                        Message::Text(text) => {
                            let pool_msg: PoolMessage<N> = serde_json::from_str(&text)?;
                            incoming_tx.send(pool_msg).await?;
                        },
                        Message::Ping(_) => {
                            ws.send(Message::Pong(vec![])).await?;
                        },
                        Message::Pong(_) => {
                            // ignore
                        },
                        _ => {
                            anyhow::bail!("Unexpected message type");
                        }
                    }
                }
                _ = heartbeat.tick() => {
                    if enable_hb {
                        ws.send(Message::Ping(vec![])).await?;
                    }
                }
            }
        }
    }

    pub async fn send(&self, msg: PoolMessage<N>) -> anyhow::Result<()> {
        self.outgoing_tx.send(msg).await?;
        Ok(())
    }

    pub async fn next(&mut self) -> anyhow::Result<PoolMessage<N>> {
        let message = self.incoming_rx.recv().await.ok_or_else(|| anyhow::anyhow!("connection closed"))?;
        Ok(message)
    }

    pub async fn next_auth_request(&mut self) -> anyhow::Result<AuthRequest> {
        let message = self.next().await?;
        let auth_request = message.auth_request().ok_or_else(|| anyhow::anyhow!("Unexpected message type"))?;
        Ok(auth_request)
    }

    pub async fn next_auth_response(&mut self) -> anyhow::Result<AuthResponse<N>> {
        let message = self.next().await?;
        let auth_response = message.auth_response().ok_or_else(|| anyhow::anyhow!("Unexpected message type"))?;
        Ok(auth_response)
    }

    pub async fn next_solution(&mut self) -> anyhow::Result<NewSolution<N>> {
        let message = self.next().await?;
        let solution = message.new_solution().ok_or_else(|| anyhow::anyhow!("Unexpected message type"))?;
        Ok(solution)
    }

    pub async fn next_task(&mut self) -> anyhow::Result<NewTask<N>> {
        let message = self.next().await?;
        let task = message.new_task().ok_or_else(|| anyhow::anyhow!("Unexpected message type"))?;
        Ok(task)
    }
}
