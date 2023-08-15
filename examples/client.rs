use aleo_mining_protocol::{AuthRequest, PoolMessage};
use anyhow::Result;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::info;
use snarkvm_console_network::{Network, Testnet3};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
#[tokio::main]
async fn main() -> Result<()> {
    // std::env::set_var("RUST_LOG", "INFO");
    env_logger::init();
    let endpoint = url::Url::parse("ws://127.0.0.1:3030")?;
    let (ws, _) = connect_async(endpoint).await?;
    let (mut outgoing, mut incoming) = ws.split();

    // 1.Prepare auth message
    let message: PoolMessage<Testnet3> = PoolMessage::AuthRequest(AuthRequest {
        username: "test123".to_string(),
    });
    send_frame(&mut outgoing, message).await?;

    // 2. Read auth response
    let message: PoolMessage<Testnet3> = read_frame(&mut incoming).await?;
    let auth_response = message.auth_response().unwrap();
    info!(
        "[aleo-mining-protocol] receive auth response: {:?}",
        auth_response
    );

    // 3. Read new task
    let message: PoolMessage<Testnet3> = read_frame(&mut incoming).await?;
    let new_task = message.new_task().unwrap();
    info!(
        "[aleo-mining-protocol] receive auth response: {:?}",
        new_task
    );

    // 4. Prove and submit solution

    Ok(())
}

pub async fn send_frame<N: Network>(
    outgoing: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    message: PoolMessage<N>,
) -> Result<()> {
    let ws_message =
        tokio_tungstenite::tungstenite::Message::text(serde_json::to_string(&message)?);
    outgoing.send(ws_message).await?;
    Ok(())
}

pub async fn read_frame<N: Network>(
    incoming: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Result<PoolMessage<N>> {
    let ws_message = incoming.next().await.unwrap()?;
    let message: PoolMessage<N> = serde_json::from_str(&ws_message.to_string())?;
    Ok(message)
}
