use aes_gcm::aead::rand_core::CryptoRngCore;
use rand::rngs::ThreadRng;
use rsa::Pkcs1v15Encrypt;
use rsa::{Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey};

pub fn sample_key_pair(rng: &mut ThreadRng) -> (RsaPrivateKey, RsaPublicKey) {
    let private_key = RsaPrivateKey::new(rng, 2048).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);
    (private_key, public_key)
}

pub fn encrypt<R: CryptoRngCore>(data: &[u8], public_key: &RsaPublicKey, rng: &mut R) -> anyhow::Result<Vec<u8>> {
    let ciphertext = public_key.encrypt(rng, Pkcs1v15Encrypt, data)?;
    Ok(ciphertext)
}

pub fn decrypt(data: &[u8], private_key: &RsaPrivateKey) -> anyhow::Result<Vec<u8>> {
    let plaintext = private_key.decrypt(Pkcs1v15Encrypt, data)?;
    Ok(plaintext)
}

pub fn verify(data: &[u8], signature: &[u8], public_key: &RsaPublicKey) -> bool {
    public_key.verify(Pkcs1v15Sign::new_unprefixed(), data, signature).is_ok()
}

pub fn sign(data: &[u8], private_key: &RsaPrivateKey) -> anyhow::Result<Vec<u8>> {
    let signature = private_key.sign(Pkcs1v15Sign::new_unprefixed(), data)?;
    Ok(signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_key_pair() {
        let mut rng = rand::thread_rng();
        sample_key_pair(&mut rng);
    }
    #[test]
    fn test_encrypt() {
        let mut rng = rand::thread_rng();
        let (private_key, public_key) = sample_key_pair(&mut rng);
        let data = b"hello world".to_vec();
        let enc_data = encrypt(&data, &public_key, &mut rng).unwrap();
        let dec_data = decrypt(&enc_data, &private_key).unwrap();
        println!("{:?}", data);
        println!("{:?}", dec_data);
        assert_eq!(data, dec_data);
    }

    // #[test]
    // fn test_write_to_file() {
    //     let mut rng = rand::thread_rng();
    //     let (sk, pk) = sample_key_pair(&mut rng);
    //     sk.write_pkcs1_pem_file("server.pem", LineEnding::CR).unwrap();
    //     pk.write_pkcs1_pem_file("server.pub", LineEnding::CR).unwrap();
    // }
}
