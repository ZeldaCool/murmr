pub mod crypto;
pub mod net;
pub mod audio;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn connect(ip: &str) {
    //connect to specified IP
}

fn mute() {
    //Mute audio, use arc
}

fn audio_change(change: u32) {
    //Change audio, amplify using arc
}

pub struct AppState {
    mute: AtomicBool,
    volume: AtomicU8,
    screenshare: AtomicBool, //for future use
    peercount: AtomicU8,  
    socket: Mutex<Option<std::net::UdpSocket>>,
    peer: Mutex<Option<String>>,
    key: Mutex<Option<[u8; 32]>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mute: false.into(),
            volume: 50.into(),
            screenshare: false.into(),
            peercount: 0.into(),
            socket: None.into(),
            peer: None.into(),
            key: None.into(),
        }
    }
        
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
