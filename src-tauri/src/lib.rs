pub mod crypto;
pub mod net;
pub mod audio;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8};
use std::sync::atomic::Ordering::Relaxed;
use anyhow::anyhow;
use std::net::UdpSocket;
use std::thread;
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};
use crate::audio::audio_loop;
use tauri::{Builder, Manager};
use x25519_dalek::PublicKey;

//TODO: Add simple if statement on connect to check if lan or not, if not, hole punch. Add api call
//for getting public-facing IP
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn getlanip(state: tauri::State<AppState>) -> String {
    let soc = {
        let guard = state.socket.lock().unwrap();

        let soc = guard
            .as_ref()
            .ok_or("Socket not initialized");

        soc.expect("Couldn't get socket").try_clone().expect("failed to clone socket")
    }; 
    soc.local_addr().unwrap().to_string()    
}
#[tauri::command]
fn connect(ip: &str, state: tauri::State<AppState>) -> Result<(), String>{
    let socket = UdpSocket::bind("0.0.0.0").expect("Failed to bind socket");
    let send_socket = socket.try_clone().expect("Failed to clone socket");
    socket.connect(ip.to_string()).expect("Failed to connect");

    let (tx, rx) = std::sync::mpsc::channel::<Vec<f32>>();
    
    {
        let mut guard = state.socket.lock().unwrap();
        *guard = Some(socket.try_clone().expect("Failed to unwrap socket"));
    }

    //shift to oneshot channels
    let (crytx, cryrx) = std::sync::mpsc::channel::<PublicKey>();

    let (keytx, keyrx) = std::sync::mpsc::channel::<[u8;32]>();

    let ring = HeapRb::<f32>::new(48000 * 5);
    let (mut producer, mut consumer) = ring.split();


    thread::spawn(move || {
       net::recv_loop(socket, producer, crytx, keyrx);     
    });

    thread::spawn(move || {
        net::send_loop(rx, send_socket, cryrx, keytx);
    });
    
    let mut mute = state.mute.clone(); 
    let mut vol = state.volume.clone();

    thread::spawn(move || {
        audio::audio_input(tx, mute, vol);
    });

    thread::spawn(move || {
        net::test_client();
    });

    thread::spawn(move || {
        audio::audio_output(consumer);
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
fn getip(state: tauri::State<AppState>) -> String {
    let soc = {
        let guard = state.socket.lock().unwrap();

        let soc = guard
            .as_ref()
            .ok_or("Socket not initialized");

        soc.expect("Couldn't get socket").try_clone().expect("failed to clone socket")
    }; 
    net::stun::stun_connect(soc).expect("No IP supplied")
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
    socket: Mutex<Option<std::net::UdpSocket>>,
    peer: Mutex<Option<String>>,
    key: Mutex<Option<[u8; 32]>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mute: Arc::new(AtomicBool::new(false.into())),
            volume: Arc::new(AtomicU8::new(50.into())),
            screenshare: Arc::new(AtomicBool::new(false.into())),
            peercount: Arc::new(AtomicU8::new(0.into())),
            socket: None.into(),
            peer: None.into(),
            key: None.into(),
        }
    }
        
}

pub fn run_tauri() -> Result<(), String>{
    Builder::default()
    .setup(|app| {
      app.manage(AppState::default());
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![connect, toggle_mic, audio_change, set_key])
    .run(tauri::generate_context!())
    .unwrap();
    Ok(())
}

