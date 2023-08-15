use std::{env, net::SocketAddr, sync::Arc};

use aleo_mining_protocol::{AuthResponse, NewTask, PoolMessage};
use anyhow::{ensure, Result};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::*;
use snarkvm::prelude::{
    coinbase::{EpochChallenge, ProverSolution},
    Address, Network,
};

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
pub struct PoolState<N: Network> {
    address: Address<N>,
    epoch_challenge: EpochChallenge<N>,
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "INFO");
    env_logger::init();
    let addr = "127.0.0.1:3030";
    let address =
        Address::from_str("aleo1rhgdu77hgyqd3xjj8ucu3jj9r2krwz6mnzyd80gncr5fxcwlh5rsvzp9px")
            .unwrap();
    let epoch_challenge = EpochChallenge::new(0, Default::default(), (1 << 12) - 1).unwrap();
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    let state = Arc::new(PoolState {
        address,
        epoch_challenge,
    });
    info!("Listening on: {}", addr);
    while let Ok((stream, peer)) = listener.accept().await {
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_connection::<Testnet3>(stream, peer, state).await {
                error!("Handle connection error: {err}");
            }
        });
    }
}

async fn handle_connection<N: Network>(
    stream: TcpStream,
    peer: SocketAddr,
    state: Arc<PoolState<N>>,
) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New websocket client: {}", peer);
    let (mut outgoing, mut incoming) = ws_stream.split();
    // 1. Wait for auth request
    let message: PoolMessage<N> = read_frame(&mut incoming).await?;
    ensure!(matches!(message, PoolMessage::AuthRequest(..)));
    let _ = message.auth_request().unwrap();

    // 2. Send auth response
    let message = PoolMessage::AuthResponse(AuthResponse {
        result: true,
        address: state.address.clone(),
        message: None,
    });
    send_frame(&mut outgoing, message).await?;
    // 3. Notify current epoch challenge
    let message = PoolMessage::NewTask(NewTask {
        epoch_challenge: state.epoch_challenge.clone(),
        difficulty: 0,
    });
    send_frame(&mut outgoing, message).await?;

    // 4. Receive solution
    let message: PoolMessage<N> = read_frame(&mut incoming).await?;
    matches!(message, PoolMessage::NewSolution(..));
    Ok(())
}

pub async fn send_frame<N: Network>(
    outgoing: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    message: PoolMessage<N>,
) -> Result<()> {
    let ws_message =
        tokio_tungstenite::tungstenite::Message::text(serde_json::to_string(&message)?);
    outgoing.send(ws_message).await?;
    Ok(())
}

pub async fn read_frame<N: Network>(
    incoming: &mut SplitStream<WebSocketStream<TcpStream>>,
) -> Result<PoolMessage<N>> {
    let ws_message = incoming.next().await.unwrap()?;
    let message: PoolMessage<N> = serde_json::from_str(&ws_message.to_string())?;
    Ok(message)
}
