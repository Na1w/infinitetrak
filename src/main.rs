mod audio;
mod core;
mod ui;

use crate::audio::TrackerEngine;
use crate::core::SharedState;
use crate::ui::{App, run_app};
use cpal::SizedSample;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use infinitedsp_core::core::channels::Mono;
use infinitedsp_core::core::frame_processor::FrameProcessor;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting InfiniTrak...");

    // Setup audio
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let config = device.default_output_config()?;

    // Create shared state
    let sample_rate = config.sample_rate() as f32;
    let state = Arc::new(Mutex::new(SharedState::new(120.0, sample_rate)));

    // Start audio stream
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => run_audio::<f32>(&device, &config.into(), state.clone())?,
        cpal::SampleFormat::I16 => run_audio::<i16>(&device, &config.into(), state.clone())?,
        cpal::SampleFormat::U16 => run_audio::<u16>(&device, &config.into(), state.clone())?,
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    };

    stream.play()?;

    // Start TUI
    let app = App::new(state);
    run_app(app)?;

    Ok(())
}

fn run_audio<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    state: Arc<Mutex<SharedState>>,
) -> Result<cpal::Stream, Box<dyn std::error::Error>>
where
    T: cpal::Sample + cpal::FromSample<f32> + SizedSample,
{
    let sample_rate = config.sample_rate as f32;
    let channels = config.channels as usize;

    let mut engine = TrackerEngine::new(sample_rate, state);

    let mut sample_index = 0u64;
    let mut processing_buffer = Vec::with_capacity(1024);
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let frames = data.len() / channels;

            if processing_buffer.len() < frames {
                processing_buffer.resize(frames, 0.0);
            }

            let buffer_slice = &mut processing_buffer[0..frames];

            FrameProcessor::<Mono>::process(&mut engine, buffer_slice, sample_index);

            sample_index += frames as u64;

            for (i, frame) in data.chunks_mut(channels).enumerate() {
                let sample_val = buffer_slice[i];
                let sample = cpal::Sample::from_sample(sample_val);
                for out_sample in frame.iter_mut() {
                    *out_sample = sample;
                }
            }
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}
