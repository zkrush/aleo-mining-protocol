use crate::connection::WebSocketStream;
use anyhow::{anyhow, ensure, Context};
use futures_util::{SinkExt, StreamExt};
use ring::rand::SystemRandom;
use rsa::pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey};
use tokio_tungstenite::tungstenite::Message;

pub mod aes_algorithm;
pub mod ec25519_algorithm;
pub mod hkdf_algorithm;
pub mod rsa_algorithm;

pub struct SharedKey {
    pub key: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl SharedKey {
    pub fn encrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        aes_algorithm::encrypt_data(data, &self.key, &self.nonce)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> anyhow::Result<Vec<u8>> {
        aes_algorithm::decrypt_data(ciphertext, &self.key, &self.nonce)
    }
}

pub async fn handshake_as_client(mut conn: WebSocketStream) -> anyhow::Result<(WebSocketStream, SharedKey)> {
    let static_server_pk = rsa::RsaPublicKey::from_pkcs1_pem(include_str!("../../../server.pub")).context("parse pub")?;
    let mut rng = ring::rand::SystemRandom::new();
    let mut trng = rand::rngs::OsRng;
    let (sk, pk) = ec25519_algorithm::sample_key_pair(&mut rng)?;

    // 1. Send a encrypted public key
    let encrypted_pk = rsa_algorithm::encrypt(pk.as_ref(), &static_server_pk, &mut trng)?;

    conn.send(Message::Binary(encrypted_pk)).await?;

    // 2. Receive a public key and signature
    let peer_pk_data = conn.next().await.ok_or(anyhow!("stream closed"))??.into_data();
    let signature = conn.next().await.ok_or(anyhow!("stream closed"))??.into_data();
    // 3. Verify the public key and signature
    ensure!(
        rsa_algorithm::verify(&peer_pk_data, &signature, &static_server_pk),
        "failed to verify signature"
    );

    let peer_pk = ring::agreement::UnparsedPublicKey::new(&ring::agreement::X25519, peer_pk_data.as_ref());
    // 4. Compute the shared key
    let share_key = ec25519_algorithm::compute_share_key(sk, peer_pk)?;

    let mut share_key = hkdf_algorithm::derive_key(share_key.as_ref(), "zkrush111".as_bytes(), &[], 32 + 12)?;
    let nonce = share_key.split_off(32);
    let shared_key = SharedKey { key: share_key, nonce };

    Ok((conn, shared_key))
}

pub async fn handshake_as_server(mut conn: WebSocketStream) -> anyhow::Result<(WebSocketStream, SharedKey)> {
    // TODO: Don't compile this when build client
    let static_server_sk = rsa::RsaPrivateKey::from_pkcs1_pem(include_str!("../../../server.pem"))?;
    let mut rng = SystemRandom::new();

    // 1. Receive a encrypted public key
    let encrypted_peer_pk = conn.next().await.ok_or(anyhow!("stream closed"))??.into_data();
    let peer_pk = rsa_algorithm::decrypt(&encrypted_peer_pk, &static_server_sk)?;

    // 2. Send a public key and the signature of the public key
    let (sk, pk) = ec25519_algorithm::sample_key_pair(&mut rng)?;
    let signature = rsa_algorithm::sign(pk.as_ref(), &static_server_sk)?;
    conn.send(Message::Binary(pk.as_ref().to_vec())).await?;
    conn.send(Message::Binary(signature)).await?;

    // 3. Compute the shared key
    let peer_pk = ring::agreement::UnparsedPublicKey::new(&ring::agreement::X25519, peer_pk.as_ref());
    let share_key = ec25519_algorithm::compute_share_key(sk, peer_pk)?;
    let mut share_key = hkdf_algorithm::derive_key(share_key.as_ref(), "zkrush111".as_bytes(), &[], 32 + 12)?;
    let nonce = share_key.split_off(32);
    let shared_key = SharedKey { key: share_key, nonce };
    Ok((conn, shared_key))
}
