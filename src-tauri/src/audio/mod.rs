use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};
use bytemuck::cast_slice;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::atomic::Ordering::Relaxed;
use crate::AppState;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8};

pub fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {err}");
}

pub fn audio_input(tx: Sender<Vec<f32>>, mute: Arc<AtomicBool>, vol: Arc<AtomicU8>, running: Arc<AtomicBool>) {
    let host = cpal::default_host();
    let inputdev = host.default_input_device();
    let config: cpal::StreamConfig = inputdev.clone().expect("failed to get device.").default_input_config().expect("Failed to get config.").into();
    
    const SIZE: usize = 960;

    let mut buffer = [0f32; SIZE];
    let mut index = 0;

    let inputfn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            //println!("Sending data of len: {}", data.len());
            for &sample in data {
                if !mute.load(Relaxed) { 
                     buffer[index] = sample;
                     index += 1;

                     if index == SIZE {
                        let packet = buffer.to_vec();
                        let _ = tx.send(packet);
                        index = 0;
                    }
                }
            }
    };

    let input_stream = inputdev.as_ref().expect("Got Option::None").build_input_stream(&config, inputfn, err_fn, None).expect("Failure when trying to build input stream.");

    input_stream.play().expect("Failed to start input.");

    loop {
        if running.load(std::sync::atomic::Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        } else {
            drop(input_stream);
            break;
        }
    }

}

pub fn audio_output(mut consumer: impl ringbuf::traits::Consumer<Item = f32> + std::marker::Send + 'static, running: Arc<AtomicBool>) {
    let host = cpal::default_host();
    let outdev = host.default_output_device();

    let config: cpal::StreamConfig = outdev.clone().expect("failed to get device.").default_output_config().expect("Failed to get config.").into();
    
   
    let outputfn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        //println!("speaker request: {}", data.len());
        for sample in data {
            *sample = consumer.try_pop().unwrap_or(0.0);
        }
    };

    
    let output_stream = outdev.as_ref().expect("Got Option::None").build_output_stream(&config, outputfn, err_fn, None).expect("Failure when trying to build output stream.");

    output_stream.play().expect("Failed to start output");

    loop {
        if running.load(std::sync::atomic::Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        } else {
            drop(output_stream);
            break;
        }
    }

}
