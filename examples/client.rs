use aleo_mining_protocol::client::Client;
use aleo_mining_protocol::AuthRequest;

use anyhow::Result;

use log::info;
use serde_json::json;
use snarkvm::console::network::Testnet3;

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "INFO");
    env_logger::init();

    let mut client: Client<Testnet3> = Client::connect("ws://127.0.0.1:3030").await?;
    // 1.Send auth message
    info!("[aleo-mining-protocol] send auth message");
    let auth_response = client
        .auth(AuthRequest {
            username: "test123".to_string(),
            metadata: json!({"machine_name": "test123"}),
        })
        .await?;
    info!("[aleo-mining-protocol] receive auth response: {:?}", auth_response);

    // 3. Read new task
    info!("[aleo-mining-protocol] subscribe new task");
    let mut rx = client.subscribe().await?;
    info!("[aleo-mining-protocol] waiting from new task");
    let new_task = rx.recv().await;
    info!("[aleo-mining-protocol] receive new task: {:?}", new_task);

    // 4. Prove and submit solution

    Ok(())
}
