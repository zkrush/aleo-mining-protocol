pub fn derive_key(secret: &[u8], salt: &[u8], info: &[u8], output_len: usize) -> anyhow::Result<Vec<u8>> {
    let salt = Some(salt);
    let mut output = vec![0u8; output_len];
    hkdf::Hkdf::<sha2::Sha256>::new(salt, secret)
        .expand(info, &mut output)
        .map_err(|err| anyhow::anyhow!("Failed to derive key: {:?}", err))?;
    Ok(output)
}
