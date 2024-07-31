use anyhow::anyhow;
use ring::agreement::{agree_ephemeral, EphemeralPrivateKey, PublicKey, UnparsedPublicKey, X25519};
use ring::rand::SystemRandom;

pub fn sample_key_pair(rng: &mut SystemRandom) -> anyhow::Result<(EphemeralPrivateKey, PublicKey)> {
    let sk = EphemeralPrivateKey::generate(&X25519, rng).map_err(|err| anyhow!("generate key pair error: {err}"))?;
    let pk = sk.compute_public_key().map_err(|err| anyhow!("compute public key error: {err}"))?;

    Ok((sk, pk))
}

pub fn compute_share_key(sk: EphemeralPrivateKey, peer_pk: UnparsedPublicKey<&[u8]>) -> anyhow::Result<Vec<u8>> {
    let shared1: Vec<u8> = agree_ephemeral(sk, &UnparsedPublicKey::new(&X25519, peer_pk.as_ref()), |key_material| {
        key_material.to_vec()
    })
    .map_err(|err| anyhow!("compute share key error: {}", err))?;
    Ok(shared1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::agreement::UnparsedPublicKey;

    #[test]
    fn test_sample_key_pair() {
        let mut rng = SystemRandom::new();
        sample_key_pair(&mut rng).unwrap();
    }

    #[test]
    fn test_share_key() {
        let mut rng = SystemRandom::new();
        let (sk1, pk1) = sample_key_pair(&mut rng).unwrap();
        let (sk2, pk2) = sample_key_pair(&mut rng).unwrap();
        let shared1: Vec<u8> = compute_share_key(sk1, UnparsedPublicKey::new(&X25519, pk2.as_ref())).unwrap();
        let shared2: Vec<u8> = compute_share_key(sk2, UnparsedPublicKey::new(&X25519, pk1.as_ref())).unwrap();
        assert_eq!(shared1, shared2);
    }
}
