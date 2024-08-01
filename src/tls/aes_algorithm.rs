use aes_gcm::aead::Aead;
use aes_gcm::{KeyInit, Nonce};
use anyhow::anyhow;

pub fn encrypt_data(data: &[u8], key: &[u8], nonce: &[u8]) -> anyhow::Result<Vec<u8>> {
    let cipher = aes_gcm::Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce);
    let ciphertext = cipher
        .encrypt(nonce, data.as_ref())
        .map_err(|err| anyhow!("encrypt error: {err}"))?;
    Ok(ciphertext)
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> anyhow::Result<Vec<u8>> {
    let nonce = Nonce::from_slice(nonce);
    let cipher = aes_gcm::Aes256Gcm::new(key.into());
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|err| anyhow!("decrypt error: {err}"))?;
    Ok(plaintext)
}
