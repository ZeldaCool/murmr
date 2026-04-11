use std::net::UdpSocket;
use std::thread;
use bytemuck::cast_slice;
use std::sync::mpsc::{Sender, Receiver};
use crate::crypto::*;
use std::collections::BTreeMap;
use x25519_dalek::PublicKey; 


pub fn test_client() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:5000")?;
    let mut buf = [0u8; 4096];

    loop {
        let (bytes_received, src_addr) = socket.recv_from(&mut buf)?;
        //println!("Recieved {} bytes", bytes_received);
        socket.send_to(&buf[..bytes_received], src_addr)?;
    }

    Ok(())
}
pub fn seri_packet_audio(audio: Vec<f32>, kind: u8, seqnum: u16) -> Vec<u8> {
    
    let slice = cast_slice(&audio);
    
    let mut buf = Vec::with_capacity(4 + slice.len());

    buf.push(kind);
    buf.push(0); //padding
    buf.extend_from_slice(&seqnum.to_le_bytes());

    buf.extend_from_slice(slice);

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
pub fn send_loop(rx: Receiver<Vec<f32>>, soc: UdpSocket) {
    //let key = key_exchange(soc.try_clone().expect("failed to clone")).unwrap();
    let mut counter: u16 = 0;
    let key = genpub();
    let packet = seri_packet_crypto(key);

    soc.send(&packet);

    for r in rx {        
       counter+=1;
       //println!("Sending {} bytes", r.len());
       let to_send = seri_packet_audio(r, 1, counter);
       soc.send(&to_send).expect("Failed to send.");
    }

}


pub fn recv_loop(soc: UdpSocket, mut producer: impl ringbuf::traits::Producer<Item = f32>) {
    let mut buf = [0u8; 4096];

    //packet outline should look like [type(1)][seqnum(2)][audio(960)]

   let mut jitbuff: BTreeMap<u16, Vec<f32>> = BTreeMap::new();
   let mut expected_next: Option<u16> = Some(1);
   let max_wait = 5;
    loop {
        match soc.recv(&mut buf) {
            Ok(len) => {
                if (len - 4) % 4 != 0 { continue; }

                if buf[0] == 0 {
                    
                } else { 
                    let samples: Vec<f32> = cast_slice(&buf[4..len]).to_vec();

                    let seqnum = u16::from_le_bytes([buf[2], buf[3]]);

                    jitbuff.insert(seqnum, samples.clone());


                    if let Some(seq) = expected_next {
                        if let Some(frame) = jitbuff.remove(&seq) {
                            for s in samples {
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
