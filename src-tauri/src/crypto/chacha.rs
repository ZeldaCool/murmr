use chacha20poly1305::{
    aead::{Aead, AeadCore},
    XChaCha20Poly1305, Nonce, KeyInit, Key
};
use rand_core::OsRng;

pub fn nonce_gen() -> [u8; 24] {
    XChaCha20Poly1305::generate_nonce(&mut OsRng).into()
}

pub fn chacha(key: [u8; 32], nonce: [u8; 24], content: Vec<u8> ) -> anyhow::Result<Vec<u8>> {
    let enckey = Key::from_slice(&key);
    let cipher = XChaCha20Poly1305::new(enckey);
    let enc = cipher.encrypt((&nonce).into(), content.as_ref()).expect("Failed to encrypt audio");
    Ok(enc)
}
