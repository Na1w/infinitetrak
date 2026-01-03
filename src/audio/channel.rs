use super::voice::SynthVoice;
use crate::core::Instrument;
use infinitedsp_core::core::channels::Mono;
use infinitedsp_core::core::frame_processor::FrameProcessor;

pub struct Channel {
    voice: SynthVoice,
    mix_buffer: Vec<f32>,
    current_instrument: Instrument,
    pub last_key: u8,
}

impl Channel {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            voice: SynthVoice::new(sample_rate),
            mix_buffer: Vec::with_capacity(1024),
            current_instrument: Instrument::default(),
            last_key: 0,
        }
    }

    pub fn trigger_note(&mut self, freq: f32, instrument: Option<&Instrument>) {
        if let Some(inst) = instrument {
            self.current_instrument = inst.clone();
        }
        self.voice.release();
        self.voice
            .update_params(freq, Some(&self.current_instrument));
        self.voice.trigger();
    }

    pub fn legato_note(&mut self, freq: f32, instrument: Option<&Instrument>) {
        if let Some(inst) = instrument {
            self.current_instrument = inst.clone();
        }
        self.voice
            .update_params(freq, Some(&self.current_instrument));
    }

    pub fn release(&mut self) {
        self.voice.release();
    }

    pub fn silence(&mut self) {
        self.voice.release();
        self.last_key = 0;
    }

    pub fn process(&mut self, buffer_len: usize, sample_index: u64) -> &[f32] {
        if self.mix_buffer.len() < buffer_len {
            self.mix_buffer.resize(buffer_len, 0.0);
        }

        let slice = &mut self.mix_buffer[0..buffer_len];
        slice.fill(0.0);

        FrameProcessor::<Mono>::process(&mut self.voice, slice, sample_index);

        slice
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        FrameProcessor::<Mono>::set_sample_rate(&mut self.voice, sample_rate);
    }
}
