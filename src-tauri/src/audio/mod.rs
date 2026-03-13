use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};
use std::sync::mpsc::{channel, Sender, Receiver};


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
            producer.try_push(sample);
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

    println!("Playing for 10 seconds... ");
    std::thread::sleep(std::time::Duration::from_secs(10));
    drop(input_stream);
    drop(output_stream);
    println!("Done!");

    Ok(())
}

pub fn audio_input(tx: Sender<Vec<f32>> ) -> anyhow::Result<()> {
    let host = cpal::default_host();
    let inputdev = host.default_input_device();

    let config: cpal::StreamConfig = inputdev.clone().expect("failed to get device.").default_input_config()?.into();

    let inputfn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let to_send = data.to_vec();
            tx.send(to_send);
    };

    Ok(())
}
