// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//fn main() {
//    murmr_lib::run()
//}
//

mod audio;
mod net;
use tokio::sync::mpsc::{Sender, Receiver};
use crate::audio::audio_loop; 

#[tokio::main]
async fn main() -> anyhow::Result<()> {
        audio_loop()?;

        Ok(())
}
