// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//fn main() {
//    murmr_lib::run()
//}
//

mod audio;
mod net;
mod crypto;
mod codec;
use std::net::UdpSocket;
use std::thread;
use murmr_lib::{AppState, run_tauri};


#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() {
    run_tauri().expect("Failed to setup tauri");
}
