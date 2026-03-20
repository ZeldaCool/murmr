use std::net::UdpSocket;
use x25519_dalek::{EphemeralSecret, PublicKey};
use rand_core::OsRng;
use sha2::Sha256;
use hkdf::Hkdf;

pub mod chacha;

pub fn key_exchange(soc: UdpSocket) -> anyhow::Result<[u8; 32]>  /*change to return key, [u8; 42]???*/ {
    let mut buf = [0u8; 4096];
    let secret = EphemeralSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);
    // send here!
    let _ = soc.send(public.as_bytes());
    // now, wait to recieve... (debug as needed later)
    let _ = soc.recv_from(&mut buf)?;
    let slice: [u8; 32] = buf[..32].try_into().map_err(|_| anyhow::anyhow!("Failed to make slice"))?;
    
    let pub_key = PublicKey::from(slice);
    let shared_secret = secret.diffie_hellman(&pub_key);

    //Don't worry, we'll derive from session later.
    let salt = b"Murmr v1 crypto";
    let info = b"Testing, testing, 123";
    let hk = Hkdf::<Sha256>::new(Some(&salt[..]), shared_secret.as_bytes());

    let mut key = [0u8; 32];
    hk.expand(info, &mut key);


    Ok(key)
} 


