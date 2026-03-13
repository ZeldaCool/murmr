use std::net::UdpSocket;
use std::thread;
use bytemuck::cast_slice;
use std::sync::mpsc::{Sender, Receiver};


pub fn test_client() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:5000")?;



    Ok(())
}

pub fn send_loop(rx: Receiver<Vec<f32>>, soc: UdpSocket) {
    for r in rx {
       let bytes = cast_slice(&r);
        
       soc.send(bytes).expect("Failed to send.");
    }

}

pub fn recv_loop(tx: Sender<Vec<f32>>, soc: UdpSocket, mut producer: impl ringbuf::traits::Producer<Item = f32>) {
    let mut buf = [0u8; 4096];

    match soc.recv(&mut buf) {
        Ok(len) => {
            let samples: &[f32] = cast_slice(&buf[..len]);
            
            for &s in samples {
                let _ = producer.try_push(s);
            }
        },
        Err(e) => eprintln!("Error: {}", e),
    };
}
