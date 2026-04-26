use tokio::net::UdpSocket;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use bytemuck::cast_slice;
use std::sync::mpsc::{Sender, Receiver};
use crate::crypto::*;
use crate::codec::*;
use std::collections::BTreeMap;
use x25519_dalek::PublicKey; 
use std::sync::Arc;
pub mod stun;


pub async fn test_client(running: Arc<AtomicBool>) -> anyhow::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:5000").await?;
    let mut buf = [0u8; 4096];

    while running.load(Ordering::Relaxed) {
        let (bytes_received, src_addr) = socket.recv_from(&mut buf).await?;
        //println!("Recieved {} bytes", bytes_received);
        socket.send_to(&buf[..bytes_received], src_addr).await?;
    }

    Ok(())
}
pub fn seri_packet_audio(audio: Vec<f32>, kind: u8, seqnum: u16, key: [u8; 32]) -> Vec<u8> {
    
    let first_slice = f32_to_i16(audio);
    let aud = linear_to_alaw(first_slice);
    let nonce = chacha::nonce_gen(); 
    let enc_packets = chacha::chacha(key, nonce, &aud).unwrap();


    let mut buf = Vec::new();

    buf.push(kind);
    buf.push(0); //padding
    buf.extend_from_slice(&seqnum.to_le_bytes());
    buf.extend_from_slice(&nonce);
    buf.extend_from_slice(&enc_packets);

    buf

       
}
pub fn seri_packet_crypto(key: PublicKey) -> Vec<u8>{
    let slice = key.as_bytes();
    let mut buf = Vec::with_capacity(2 + slice.len());

    buf.push(0);
    buf.push(0);
    buf.extend_from_slice(slice);

    buf
}

pub async fn send_loop(rx: Receiver<Vec<f32>>, soc: Arc<UdpSocket>, cryptrx: Receiver<PublicKey>, keytx: Sender<[u8; 32]>, running: Arc<AtomicBool>) {
    //let key = key_exchange(soc.try_clone().expect("failed to clone")).unwrap();
    let mut counter: u16 = 0;
    let (key, secret) = genpub();
    let packet = seri_packet_crypto(key);
    soc.send(&packet).await.expect("Failed to send encryption packet");

    println!("Getting key");
    let pubkey = cryptrx.recv().expect("Failed to get key.");
    println!("Got key");
    let enc = compute_key(pubkey, secret, key);
    
    keytx.send(enc).expect("Failed to send encrypted packet.");

    for r in rx {
       if !running.load(std::sync::atomic::Ordering::Relaxed) {
           break;
       } else {  
           counter+=1;
           //println!("Sending {} bytes", r.len());
           let to_send = seri_packet_audio(r, 1, counter, enc);
           soc.send(&to_send).await.expect("Failed to send.");
        }
    }

}


pub async fn recv_loop(soc: Arc<UdpSocket>, mut producer: impl ringbuf::traits::Producer<Item = f32>, tx: Sender<PublicKey>, keyrx: Receiver<[u8; 32]>, running: Arc<AtomicBool>) {
    let mut buf = [0u8; 4096];

    //packet outline should look like [type(1)][seqnum(2)][audio(960)]

   let mut jitbuff: BTreeMap<u16, Vec<f32>> = BTreeMap::new();
   let mut expected_next: Option<u16> = Some(1);
   let max_wait = 5;
   let mut key: Option<[u8; 32]> = None;
    while running.load(std::sync::atomic::Ordering::Relaxed) {
        match soc.recv(&mut buf).await {
            Ok(len) => {

                if &buf[..len] == b"GOODBYE" {
                    break;
                }

                if len < 28{
                    continue;
                }

                if buf[0] == 0 {
                   let bytes: [u8; 32] = buf[2..34].try_into().expect("Failed to get key.");
                   let pubkey = PublicKey::from(bytes);
                   tx.send(pubkey); 
                   key = Some(keyrx.recv().expect("Failed to get key."));
                } else { 
                    let nonce: [u8; 24] = buf[4..28].try_into().expect("Failed to convert nonce");
                    let samples = chacha::decrypt(key.clone().expect("Confusion."), nonce, &buf[28..len]);
                   
                    let seqnum = u16::from_le_bytes([buf[2], buf[3]]);

                    jitbuff.insert(seqnum, samples.clone());


                    if let Some(seq) = expected_next {
                        if let Some(frame) = jitbuff.remove(&seq) {
                            for s in frame {
                                let _ = producer.try_push(s);
                            }
                            expected_next = Some(seq.wrapping_add(1));
                        } else if jitbuff.len() > max_wait {
                            for _ in 0..480{
                                let _ = producer.try_push(0.0);
                            }
                            expected_next = Some(seq.wrapping_add(1));
                    }
                }
            }


               
                           },
            Err(e) => eprintln!("Error: {}", e),
        };
    }
}
