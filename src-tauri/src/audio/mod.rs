use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};
use bytemuck::cast_slice;
use std::sync::mpsc::{channel, Sender, Receiver};
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering::Relaxed;
use crate::AppState;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8};

pub fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {err}");
}

pub fn audio_loop() -> anyhow::Result<()>  {
    let host = cpal::default_host();
    let inputdev = host.default_input_device();
    let outputdev = host.default_output_device();

    //let inputconf: cpal::StreamConfig = inputdev.expect("failed to get input device.").default_input_config()?.into();
    //let outputconf: cpal::StreamConfig = outputdev.expect("failed to get input device.").default_output_config()?.into();

    let config: cpal::StreamConfig = inputdev.clone().expect("failed to get device(s).").default_input_config()?.into();

    let ring = HeapRb::<f32>::new(48000);
    let (mut producer, mut consumer) = ring.split();


    let inputfn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
         for &sample in data {
            let _ = producer.try_push(sample);
        }
    };

    let outputfn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        for sample in data {
            *sample = consumer.try_pop().unwrap_or(0.0);
        }
    };

    let input_stream = inputdev.as_ref().expect("Got Option::None").build_input_stream(&config, inputfn, err_fn, None)?;
    let output_stream = outputdev.as_ref().expect("Got Option::None").build_output_stream(&config, outputfn, err_fn, None)?;

    input_stream.play()?;
    output_stream.play()?;

    println!("Playing... ");
    std::thread::sleep(std::time::Duration::from_secs(999));
    drop(input_stream);
    drop(output_stream);
    println!("Done!");

    Ok(())
}


#[derive(Serialize, Deserialize, Debug)]
pub struct PacketLayout {
    //header portion
    payloadkind: u8,
    //seqnum: u16,
    //for now, just payloadkind. I'm lazy, alright??????
    //nonce: [u8; 24],
    //payload(CRAZY NOT LIKE THAT'S VAR NAME...)
    payload: Vec<f32>, 

}

pub fn audio_input(tx: Sender<Vec<f32>>, mute: Arc<AtomicBool>, vol: Arc<AtomicU8>) {
    let host = cpal::default_host();
    let inputdev = host.default_input_device();
    let config: cpal::StreamConfig = inputdev.clone().expect("failed to get device.").default_input_config().expect("Failed to get config.").into();
    
    const SIZE: usize = 960;

    let mut send_vec = Vec::with_capacity(SIZE);

    let inputfn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            //println!("Sending data of len: {}", data.len());
            for sample in data {
                if !mute.load(Relaxed){ 
                    send_vec.push(*sample);

                    if send_vec.len() == SIZE {
                        let x = send_vec.clone();
                        let _ = tx.send(x);
                        send_vec.clear();
                    }
}
                }
    };

    let input_stream = inputdev.as_ref().expect("Got Option::None").build_input_stream(&config, inputfn, err_fn, None).expect("Failure when trying to build input stream.");

    input_stream.play().expect("Failed to start input.");

    std::thread::sleep(std::time::Duration::from_secs(999));
}

pub fn audio_output(mut consumer: impl ringbuf::traits::Consumer<Item = f32> + std::marker::Send + 'static) {
    let host = cpal::default_host();
    let outdev = host.default_output_device();

    let config: cpal::StreamConfig = outdev.clone().expect("failed to get device.").default_output_config().expect("Failed to get config.").into();
    
   
    let outputfn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        //println!("speaker request: {}", data.len());
        for sample in data {
            *sample = consumer.try_pop().unwrap_or(0.0) * 3.0;
        }
    };

    
    let output_stream = outdev.as_ref().expect("Got Option::None").build_output_stream(&config, outputfn, err_fn, None).expect("Failure when trying to build output stream.");

    output_stream.play().expect("Failed to start output");

    std::thread::sleep(std::time::Duration::from_secs(999));
}
