use infinitedsp_core::core::frame_processor::FrameProcessor;
use crate::core::Instrument;
use super::voice::SynthVoice;

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
        self.voice.update_params(freq, Some(&self.current_instrument));
        self.voice.trigger();
    }

    pub fn legato_note(&mut self, freq: f32, instrument: Option<&Instrument>) {
        if let Some(inst) = instrument {
            self.current_instrument = inst.clone();
        }
        self.voice.update_params(freq, Some(&self.current_instrument));
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

        self.voice.process(slice, sample_index);

        slice
    }
}
