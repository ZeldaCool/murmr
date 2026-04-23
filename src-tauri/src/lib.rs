pub mod crypto;
pub mod net;
pub mod audio;
pub mod codec;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8};
use std::sync::atomic::Ordering::Relaxed;
use anyhow::anyhow;
use std::net::UdpSocket;
use std::net::Ipv4Addr;
use std::thread;
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};
use std::str::FromStr;
use crate::net::stun::*;
use tauri::{Builder, Manager};
use x25519_dalek::PublicKey;


#[tauri::command]
fn getlanip(state: tauri::State<AppState>) -> String {
    let soc = state.socket.clone();
    soc.local_addr().unwrap().to_string()    
}

#[tauri::command]
fn leave(state: tauri::State<AppState>) {
    let soc = state.socket.clone();

    let peer_addr = state.peer.lock().unwrap().clone();

    if let Some(peer_addr) = peer_addr {
        soc.send_to(b"GOODBYE", peer_addr);
    }
    state.running.store(false, Relaxed);
}



#[tauri::command]
fn connect(ip: &str, state: tauri::State<AppState>) -> Result<(), String> {
    let socket = state.socket.clone(); 
    
    *state.peer.lock().unwrap() = Some(ip.to_string());
    state.running.store(true, Relaxed);

    let addr = ip
        .trim()
        .parse::<std::net::SocketAddr>()
        .map_err(|_| format!("Invalid socket address (ip:port expected): {}", ip))?;

    let ip_addr = match addr.ip() {
        std::net::IpAddr::V4(v4) => v4,
        _ => return Err("IPv6 not supported".into()),
    };

    let hole = socket.clone();
    if !is_lan(ip_addr) {
        hole_punch(hole, ip.to_string());
    }

    let send_socket = socket.clone();
    let recv_socket = socket.clone();
    socket.connect(addr).expect("Failed to connect");

    let running_recv = state.running.clone();
    let running_send = state.running.clone();
    let running_input = state.running.clone();
    let running_output = state.running.clone();


    let (tx, rx) = std::sync::mpsc::channel::<Vec<f32>>();
    
    //shift to oneshot channels
    let (crytx, cryrx) = std::sync::mpsc::channel::<PublicKey>();

    let (keytx, keyrx) = std::sync::mpsc::channel::<[u8;32]>();

    let ring = HeapRb::<f32>::new(48000 * 5);
    let (mut producer, mut consumer) = ring.split();


    thread::spawn(move || {
       net::recv_loop(recv_socket, producer, crytx, keyrx, running_recv);     
    });

    thread::spawn(move || {
        net::send_loop(rx, send_socket, cryrx, keytx, running_send);
    });
    
    let mut mute = state.mute.clone(); 
    let mut vol = state.volume.clone();

    thread::spawn(move || {
        audio::audio_input(tx, mute, vol, running_input);
    });

    thread::spawn(move || {
        net::test_client();
    });

    thread::spawn(move || {
        audio::audio_output(consumer, running_output);
    });

    Ok(())
 
}

#[tauri::command]
fn toggle_mic(state: tauri::State<AppState>) {
    let current = state.mute.load(Relaxed);
    let new = !current;

    state.mute.store(new, Relaxed);
}

#[tauri::command]
fn getip(state: tauri::State<AppState>) -> Result<String, String> {
    let soc = state.socket.clone();
    net::stun::stun_connect(soc).ok_or("STUN failed".to_string())
}

#[tauri::command]
fn audio_change(volume: u8, state: tauri::State<AppState>) -> Result<(), String> {
    if volume > 100 {
        anyhow!("Volume can't be over 100");
    }

    state.volume.store(volume, Relaxed);

    Ok(())
}

#[tauri::command]
fn set_key(state: tauri::State<AppState>, key: [u8; 32]) {
    let mut locked = state.key.lock().unwrap();
    *locked = Some(key);

}

/*#[tauri::command]
fn status() -> (bool, u8, u8) {

}*/
pub struct AppState {
    mute: Arc<AtomicBool>,
    volume: Arc<AtomicU8>,
    screenshare: Arc<AtomicBool>, //for future use
    peercount: Arc<AtomicU8>, 
    running: Arc<AtomicBool>,
    socket: Arc<std::net::UdpSocket>,
    peer: Mutex<Option<String>>,
    key: Mutex<Option<[u8; 32]>>,
    ip: Mutex<Option<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mute: Arc::new(AtomicBool::new(false.into())),
            volume: Arc::new(AtomicU8::new(50.into())),
            screenshare: Arc::new(AtomicBool::new(false.into())),
            peercount: Arc::new(AtomicU8::new(0.into())),
            running: Arc::new(AtomicBool::new(false)),
            socket: Arc::new(UdpSocket::bind("0.0.0.0:0").unwrap()).into(),
            peer: Mutex::new(None.into()),
            key: None.into(),
            ip: Mutex::new(Some(String::new())),
        }
    }
        
}

pub fn run_tauri() -> Result<(), String>{
    Builder::default()
    .setup(|app| {
      app.manage(AppState::default());
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![connect, toggle_mic, audio_change, set_key, getlanip, getip, leave])
    .run(tauri::generate_context!())
    .unwrap();
    Ok(())
}

