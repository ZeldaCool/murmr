use std::net::{UdpSocket, SocketAddr, Ipv4Addr};
use rand::prelude::*;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::Arc;

const MSG_TYPE: [u8; 2] = 0x0001_u16.to_be_bytes();
const SIZE: [u8; 2] = 0x0000_u16.to_be_bytes();
const MAGIC_COOKIE: [u8; 4] = 0x2112A442_u32.to_be_bytes();

pub fn stun_connect(soc: Arc<UdpSocket>) -> Option<String> {
    let mut rng = rand::rng();
    let mut packet = [0u8; 20];
    let mut recvbuf = [0u8; 512];
    packet[0..2].copy_from_slice(&MSG_TYPE);
    packet[2..4].copy_from_slice(&SIZE);
    packet[4..8].copy_from_slice(&MAGIC_COOKIE);

    let tid: [u8; 12] = rng.random();

    packet[8..20].copy_from_slice(&tid);

    soc.send_to(&packet, "stun.l.google.com:19302");

    let (res, src) = soc.recv_from(&mut recvbuf).expect("Error getting response.");
    
    let ip = get_ip(&recvbuf, tid, res);

    return ip
}

pub fn get_ip(buf: &[u8], tid: [u8; 12], len: usize ) -> Option<String> {
    if len < 20 {
        return None;
    }
    if &buf[8..20] != &tid {
        return None;
    }
    if &buf[4..8] != MAGIC_COOKIE {
        return None;
    }

    let mut i: usize = 20;

    while i + 4 < len {
        let attr = u16::from_be_bytes([buf[i], buf[i + 1]]);
        let attr_len = u16::from_be_bytes([buf[i + 2], buf[i + 3]]);

        let attr_start = i + 4;
        let val_end = attr_start + attr_len as usize;

        if val_end > len {
            break;
        }

        if attr == 0x0020 {
            let family = buf[attr_start + 1];

            if family == 0x01 {
                let xport = {
                    let x = u16::from_be_bytes([
                        buf[attr_start + 2],
                        buf[attr_start + 3],
                    ]);

                    let port = x ^ 0x2112;

                    port
                };

                let ip = {
                    let xorip = [
                        buf[attr_start + 4],
                        buf[attr_start + 5],
                        buf[attr_start + 6],
                        buf[attr_start + 7],
                    ];

                    let decode = [
                        xorip[0] ^ MAGIC_COOKIE[0],
                        xorip[1] ^ MAGIC_COOKIE[1],
                        xorip[2] ^ MAGIC_COOKIE[2],
                        xorip[3] ^ MAGIC_COOKIE[3],
                    ];

                    decode 
                };

                let ip_str = format!("{}.{}.{}.{}:{}", ip[0], ip[1], ip[2], ip[3], xport);
                return Some(ip_str);        
            }
        }
        let padded_len = (attr_len + 3) & !3;
        i += 4 + padded_len as usize;
    }

    None
}

pub fn hole_punch(soc: Arc<UdpSocket>, ip: String) {
    let packet = [3u8];
    let mut buf = [0u8; 3];

    let ipp: SocketAddr = ip.parse().expect("Failed to convert ip to SocketAddr");

    let start  = Instant::now();
    let dur = Duration::from_secs(5);

    while start.elapsed() < dur {
       soc.send_to(&packet, &ipp);
    
       if let Ok((len, src)) = soc.recv_from(&mut buf) {
           if src == ipp && len == packet.len() {
                break;
           }
       }
    }
}

pub fn is_lan(ip: Ipv4Addr) -> bool {
    //checks if LAN, if not, STUN & Hole punch
    let octets = ip.octets();
    
    match octets {
        [10, _, _, _] => true,

        [192, 168, _, _] => true,

        [127, _, _, _] => true,

        [172, b, _, _] if(16..=31).contains(&b) => true,

        _ => false,
    }
}
