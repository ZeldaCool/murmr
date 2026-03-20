use std::net::UdpSocket;
use std::thread;
use bytemuck::cast_slice;
use std::sync::mpsc::{Sender, Receiver};
use crate::crypto::key_exchange; 


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
pub fn seri_packet(audio: Vec<f32>, kind: u8, seqnum: u16) -> Vec<u8> {
    
    //net should serialize
    let slice = cast_slice(&audio);
    
    let mut buf = Vec::with_capacity(4 + slice.len());

    buf.push(kind);
    buf.push(0); //padding
    buf.extend_from_slice(&seqnum.to_le_bytes());

    buf.extend_from_slice(slice);

    buf

}

pub fn send_loop(rx: Receiver<Vec<f32>>, soc: UdpSocket) {
    //let key = key_exchange(soc.try_clone().expect("failed to clone")).unwrap();
    let mut counter: u16 = 0;
    for r in rx {
       counter+=1;
       //println!("Sending {} bytes", r.len());
       let to_send = seri_packet(r, 1, counter);
       soc.send(&to_send).expect("Failed to send.");
    }

}


pub fn recv_loop(soc: UdpSocket, mut producer: impl ringbuf::traits::Producer<Item = f32>) {
    let mut buf = [0u8; 4096];

    //packet outline should look like [type(1)][seqnum(2)][audio(960)]

    
    loop {
        match soc.recv(&mut buf) {
            Ok(len) => {
                if (len - 4) % 4 != 0 { continue; }

                /*if &buf[..1] == 0 {


                }*/

                let samples: &[f32] = cast_slice(&buf[4..len]);

                for &s in samples {
                    //println!("Pushing to output ringbuf");
                    let _ = producer.try_push(s);
                }
            },
            Err(e) => eprintln!("Error: {}", e),
        };
    }
}
