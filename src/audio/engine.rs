use infinitedsp_core::core::frame_processor::FrameProcessor;
use std::sync::{Arc, Mutex};
use crate::core::{SharedState, NUM_CHANNELS, ROWS_PER_PATTERN, NUM_INSTRUMENTS};
use super::channel::Channel;

pub struct TrackerEngine {
    channels: [Channel; NUM_CHANNELS],
    state: Arc<Mutex<SharedState>>,
    preview_timers: [usize; NUM_CHANNELS],
    preview_duration: usize,
    was_playing: bool,
}

impl TrackerEngine {
    pub fn new(sample_rate: f32, state: Arc<Mutex<SharedState>>) -> Self {
        let mut channels_vec = Vec::with_capacity(NUM_CHANNELS);
        for _ in 0..NUM_CHANNELS {
            channels_vec.push(Channel::new(sample_rate));
        }

        let channels: [Channel; NUM_CHANNELS] = channels_vec.try_into()
            .map_err(|_| "Failed to init channels").unwrap();

        Self {
            channels,
            state,
            preview_timers: [0; NUM_CHANNELS],
            preview_duration: (sample_rate * 0.5) as usize,
            was_playing: false,
        }
    }

    fn midi_to_freq(midi: u8) -> f32 {
        440.0 * 2.0f32.powf((midi as f32 - 69.0) / 12.0)
    }

    fn tick(&mut self) {
        let mut state = self.state.lock().unwrap();
        if !state.is_playing {
            return;
        }

        let row_idx = state.current_row;
        let row = state.pattern.rows[row_idx];

        for (i, note) in row.iter().enumerate() {
            let inst_idx = i % NUM_INSTRUMENTS;
            let instrument = Some(&state.instruments[inst_idx]);

            if note.key > 0 {
                let freq = Self::midi_to_freq(note.key);

                // Logic:
                // Channels 0-2 are Drums -> Always Retrigger
                // Channels 3+ are Synths -> Legato if same key
                let is_drum_channel = i <= 2;

                if !is_drum_channel && note.key == self.channels[i].last_key {
                    self.channels[i].legato_note(freq, instrument);
                } else {
                    self.channels[i].trigger_note(freq, instrument);
                }

                self.channels[i].last_key = note.key;

            } else {
                // Empty row behavior:
                // Empty row triggers Note Off (Release) to allow for staccato.
                // Legato is achieved by placing consecutive notes without gaps.
                self.channels[i].release();
                self.channels[i].last_key = 0;
            }
        }

        state.current_row = (state.current_row + 1) % ROWS_PER_PATTERN;
    }
}

impl FrameProcessor for TrackerEngine {
    fn process(&mut self, buffer: &mut [f32], sample_index: u64) {
        buffer.fill(0.0);
        let frames = buffer.len();

        let mut ticks_to_process = 0;
        let is_playing;

        {
            let mut state = self.state.lock().unwrap();

            if let Some((ch_idx, key)) = state.preview_request.take() {
                if ch_idx < NUM_CHANNELS {
                    if key > 0 {
                        let freq = Self::midi_to_freq(key);
                        let inst_idx = ch_idx % NUM_INSTRUMENTS;
                        let instrument = Some(&state.instruments[inst_idx]);

                        self.channels[ch_idx].trigger_note(freq, instrument);
                        self.channels[ch_idx].last_key = key;
                        self.preview_timers[ch_idx] = self.preview_duration;
                    } else {
                        self.channels[ch_idx].release();
                        self.preview_timers[ch_idx] = 0;
                    }
                }
            }

            is_playing = state.is_playing;
            if is_playing {
                state.current_tick_samples += frames;
                while state.current_tick_samples >= state.samples_per_tick {
                    state.current_tick_samples -= state.samples_per_tick;
                    ticks_to_process += 1;
                }
            }
        }

        if self.was_playing && !is_playing {
            for channel in &mut self.channels {
                channel.silence();
            }
        }
        self.was_playing = is_playing;

        if !is_playing {
            for i in 0..NUM_CHANNELS {
                if self.preview_timers[i] > 0 {
                    if self.preview_timers[i] > frames {
                        self.preview_timers[i] -= frames;
                    } else {
                        self.preview_timers[i] = 0;
                        self.channels[i].release();
                    }
                }
            }
        } else {
            for _ in 0..ticks_to_process {
                self.tick();
            }
        }

        for channel in &mut self.channels {
            let channel_out = channel.process(frames, sample_index);
            for (i, sample) in buffer.iter_mut().enumerate() {
                *sample += channel_out[i];
            }
        }

        for sample in buffer.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }
    }
}
