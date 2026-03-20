// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//fn main() {
//    murmr_lib::run()
//}
//

mod audio;
mod net;
mod crypto;
use tokio::sync::mpsc::{Sender, Receiver};
use crate::audio::audio_loop; 
use std::net::UdpSocket;
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};
use std::thread;
use tauri::{Builder, Manager};
use murmr_lib::AppState;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::default()
    .setup(|app| {
      app.manage(AppState::default());
      Ok(())
    })
    .run(tauri::generate_context!())
    .unwrap();
    let socket = UdpSocket::bind("0.0.0.0:34254")?;
    let send_socket = socket.try_clone()?;
    socket.connect("127.0.0.1:5000").expect("Failed to connect");

    let (tx, rx) = std::sync::mpsc::channel::<Vec<f32>>();

    let ring = HeapRb::<f32>::new(48000 * 5);
    let (mut producer, mut consumer) = ring.split();


    thread::spawn(move || {
       net::recv_loop(socket, producer);     
    });

    thread::spawn(move || {
        net::send_loop(rx, send_socket);
    });

    thread::spawn(move || {
        audio::audio_input(tx);
    });

    thread::spawn(move || {
        net::test_client();
    });

    std::thread::sleep(std::time::Duration::from_millis(500));
    
    audio::audio_output(consumer);

    Ok(())
}
