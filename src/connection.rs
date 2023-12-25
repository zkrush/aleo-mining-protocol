use crate::PoolMessage;
use futures_util::SinkExt;
use futures_util::{
    stream::{SplitSink as FutureSplitSink, SplitStream as FutureSplitStream},
    StreamExt,
};
use snarkvm::prelude::Network;
use std::{
    marker::PhantomData,
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream as TokioWebSocketStream};

type WebSocketStream = TokioWebSocketStream<MaybeTlsStream<TcpStream>>;
pub type SplitStream = FutureSplitStream<WebSocketStream>;
pub type SplitSink = FutureSplitSink<WebSocketStream, Message>;

pub struct Connection<N: Network> {
    outgoing: SplitSink,
    incoming: Option<SplitStream>,
    broken: AtomicBool,
    _p: PhantomData<N>,
}

impl<N: Network> Connection<N> {
    pub fn is_broken(&self) -> bool {
        self.broken.load(Ordering::SeqCst)
    }

    pub fn new(ws: WebSocketStream) -> Self {
        let (outgoing, incoming) = ws.split();
        Self {
            outgoing,
            incoming: Some(incoming),
            broken: AtomicBool::new(false),
            _p: PhantomData,
        }
    }

    pub async fn read_pool_message(&mut self) -> anyhow::Result<PoolMessage<N>> {
        if self.is_broken() {
            return Err(anyhow::anyhow!("Connection to server has broken"));
        }
        let incoming = match self.incoming.as_mut() {
            None => return Err(anyhow::anyhow!("Incoming has been taken")),
            Some(incoming) => incoming,
        };
        let pool_msg = read_pool_message_from_stream::<N>(incoming).await?;
        Ok(pool_msg)
    }

    pub async fn write_pool_message(&mut self, msg: PoolMessage<N>) -> anyhow::Result<()> {
        if self.is_broken() {
            return Err(anyhow::anyhow!("Connection to server has broken"));
        }
        let text = serde_json::to_string(&msg)?;
        self.outgoing.send(Message::Text(text)).await?;
        Ok(())
    }

    pub async fn take_stream(&mut self) -> anyhow::Result<Option<SplitStream>> {
        if self.is_broken() {
            return Err(anyhow::anyhow!("Connection to server has broken"));
        }
        Ok(self.incoming.take())
    }
}

pub async fn read_pool_message_from_stream<N: Network>(incoming: &mut SplitStream) -> anyhow::Result<PoolMessage<N>> {
    let ws_msg = match incoming.next().await {
        None => return Err(anyhow::anyhow!("Connection to server has broken")),
        Some(ws_msg) => ws_msg?,
    };
    let pool_msg = match ws_msg {
        Message::Text(text) => serde_json::from_str(&text)?,
        _ => return Err(anyhow::anyhow!("Unexpected message type")),
    };
    Ok(pool_msg)
}
