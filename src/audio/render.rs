use std::sync::{Arc, Mutex};
use hound;
use infinitedsp_core::core::frame_processor::FrameProcessor;
use crate::core::{SharedState, ROWS_PER_PATTERN};
use crate::audio::TrackerEngine;

pub fn render_to_wav(path: &str, state: &SharedState) -> Result<(), Box<dyn std::error::Error>> {
    let sample_rate = 44100.0;

    // Clone state for offline rendering
    let mut render_state = state.clone();

    // Recalculate timing for the target sample rate
    let samples_per_tick = ((sample_rate * 60.0) / (render_state.bpm * 4.0)) as usize;
    render_state.samples_per_tick = samples_per_tick;

    // Reset state for rendering
    render_state.current_row = 0;
    render_state.current_tick_samples = samples_per_tick; // Start immediately
    render_state.is_playing = true;
    render_state.preview_request = None; // Ensure no stray preview notes

    // Create engine
    let state_arc = Arc::new(Mutex::new(render_state));
    let mut engine = TrackerEngine::new(sample_rate, state_arc.clone());

    let total_samples = ROWS_PER_PATTERN * samples_per_tick;

    // Setup WAV writer
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)?;

    // Render loop
    let block_size = 512;
    let mut buffer = vec![0.0; block_size * 2];

    let mut samples_rendered = 0;

    while samples_rendered < total_samples {
        let remaining = total_samples - samples_rendered;
        let current_block_size = block_size.min(remaining);

        let mono_slice = &mut buffer[0..current_block_size];

        engine.process(mono_slice, samples_rendered as u64);

        for &sample in mono_slice.iter() {
            let amplitude = i16::MAX as f32;
            let val = (sample.clamp(-1.0, 1.0) * amplitude) as i16;
            writer.write_sample(val)?;
            writer.write_sample(val)?;
        }

        samples_rendered += current_block_size;
    }

    writer.finalize()?;
    Ok(())
}
