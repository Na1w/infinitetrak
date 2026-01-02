use super::instrument::{Instrument, ModuleConfig, WaveformType};
use super::pattern::Pattern;
use super::NUM_INSTRUMENTS;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PlayMode {
    Pattern,
    Song,
}

#[derive(Clone)]
pub struct SharedState {
    pub patterns: Vec<Pattern>,
    pub current_pattern: usize,
    pub instruments: [Instrument; NUM_INSTRUMENTS],
    pub current_row: usize,
    pub is_playing: bool,
    pub play_mode: PlayMode,
    pub bpm: f32,
    pub samples_per_tick: usize,
    pub current_tick_samples: usize,
    pub preview_request: Option<(usize, u8)>,
}

impl SharedState {
    pub fn new(bpm: f32, sample_rate: f32) -> Self {
        let samples_per_row = (sample_rate * 60.0) / (bpm * 4.0);

        let mut instruments = Vec::with_capacity(NUM_INSTRUMENTS);
        for _ in 0..NUM_INSTRUMENTS {
            instruments.push(Instrument::default());
        }

        // Channel 1 (Inst 0): Kick Drum
        instruments[0].name = "Kick".to_string();
        instruments[0].modules = vec![
            ModuleConfig::Oscillator {
                waveform: WaveformType::Sine,
                pitch_offset: 0.0,
                detune: 0.0,
                pitch_env_amount: 150.0,
                pitch_env_decay: 0.05
            },
            ModuleConfig::Filter { cutoff: 2000.0, resonance: 0.0 },
            ModuleConfig::Adsr { attack: 0.001, decay: 0.2, sustain: 0.0, release: 0.1 },
            ModuleConfig::Gain { level: 0.9 },
        ];

        // Channel 2 (Inst 1): Hihat
        instruments[1].name = "Hihat Cl".to_string();
        instruments[1].modules = vec![
            ModuleConfig::Oscillator {
                waveform: WaveformType::Noise,
                pitch_offset: 0.0,
                detune: 0.0,
                pitch_env_amount: 0.0,
                pitch_env_decay: 0.1
            },
            ModuleConfig::Filter { cutoff: 10000.0, resonance: 0.0 },
            ModuleConfig::Adsr { attack: 0.001, decay: 0.05, sustain: 0.0, release: 0.05 },
            ModuleConfig::Gain { level: 0.6 },
        ];

        // Channel 3 (Inst 2): Snare
        instruments[2].name = "Snare".to_string();
        instruments[2].modules = vec![
            ModuleConfig::Oscillator {
                waveform: WaveformType::Noise,
                pitch_offset: 0.0,
                detune: 0.0,
                pitch_env_amount: 0.0,
                pitch_env_decay: 0.1
            },
            ModuleConfig::Filter { cutoff: 3000.0, resonance: 0.2 },
            ModuleConfig::Adsr { attack: 0.001, decay: 0.15, sustain: 0.0, release: 0.1 },
            ModuleConfig::Gain { level: 0.7 },
        ];

        // Channel 4 (Inst 3): Bass
        instruments[3].name = "Bass Saw".to_string();
        instruments[3].modules = vec![
            ModuleConfig::Oscillator {
                waveform: WaveformType::Saw,
                pitch_offset: 0.0,
                detune: 0.0,
                pitch_env_amount: 0.0,
                pitch_env_decay: 0.1
            },
            ModuleConfig::Filter { cutoff: 400.0, resonance: 0.4 },
            ModuleConfig::Adsr { attack: 0.01, decay: 0.2, sustain: 0.6, release: 0.2 },
            ModuleConfig::Gain { level: 0.6 },
        ];

        // Channel 5 (Inst 4): Lead
        instruments[4].name = "Lead Sq".to_string();
        instruments[4].modules = vec![
            ModuleConfig::Oscillator {
                waveform: WaveformType::Square,
                pitch_offset: 0.0,
                detune: 0.0,
                pitch_env_amount: 0.0,
                pitch_env_decay: 0.1
            },
            ModuleConfig::Filter { cutoff: 3000.0, resonance: 0.2 },
            ModuleConfig::Adsr { attack: 0.02, decay: 0.1, sustain: 0.8, release: 0.3 },
            ModuleConfig::Gain { level: 0.5 },
        ];

        // Channel 6 (Inst 5): Pluck
        instruments[5].name = "Pluck".to_string();
        instruments[5].modules = vec![
            ModuleConfig::Oscillator {
                waveform: WaveformType::Triangle,
                pitch_offset: 0.0,
                detune: 0.0,
                pitch_env_amount: 0.0,
                pitch_env_decay: 0.1
            },
            ModuleConfig::Filter { cutoff: 2000.0, resonance: 0.0 },
            ModuleConfig::Adsr { attack: 0.001, decay: 0.3, sustain: 0.0, release: 0.3 },
            ModuleConfig::Gain { level: 0.6 },
        ];

        // Channel 7 (Inst 6): Pad
        instruments[6].name = "Pad".to_string();
        instruments[6].modules = vec![
            ModuleConfig::Oscillator {
                waveform: WaveformType::Saw,
                pitch_offset: 0.0,
                detune: 0.0,
                pitch_env_amount: 0.0,
                pitch_env_decay: 0.1
            },
            ModuleConfig::Filter { cutoff: 800.0, resonance: 0.1 },
            ModuleConfig::Adsr { attack: 0.5, decay: 0.5, sustain: 0.7, release: 1.0 },
            ModuleConfig::Gain { level: 0.4 },
        ];

        // Channel 8 (Inst 7): Acid
        instruments[7].name = "Acid".to_string();
        instruments[7].modules = vec![
            ModuleConfig::Oscillator {
                waveform: WaveformType::Saw,
                pitch_offset: 0.0,
                detune: 0.0,
                pitch_env_amount: 0.0,
                pitch_env_decay: 0.1
            },
            ModuleConfig::Filter { cutoff: 600.0, resonance: 0.8 },
            ModuleConfig::Adsr { attack: 0.01, decay: 0.2, sustain: 0.2, release: 0.1 },
            ModuleConfig::Gain { level: 0.5 },
        ];

        let instruments_array: [Instrument; NUM_INSTRUMENTS] = instruments.try_into().expect("Wrong size");

        Self {
            patterns: vec![Pattern::default()],
            current_pattern: 0,
            instruments: instruments_array,
            current_row: 0,
            is_playing: false,
            play_mode: PlayMode::Pattern,
            bpm,
            samples_per_tick: samples_per_row as usize,
            current_tick_samples: samples_per_row as usize,
            preview_request: None,
        }
    }

    pub fn load_project(&mut self, project: super::io::Project) {
        self.bpm = project.bpm;

        if !project.patterns.is_empty() {
            self.patterns = project.patterns;
        } else if let Some(p) = project.pattern {
            self.patterns = vec![p];
        } else {
            self.patterns = vec![Pattern::default()];
        }

        self.current_pattern = 0;

        let mut instruments_vec = Vec::with_capacity(NUM_INSTRUMENTS);
        for _ in 0..NUM_INSTRUMENTS {
            instruments_vec.push(Instrument::default());
        }

        for (i, inst) in project.instruments.into_iter().enumerate() {
            if i < NUM_INSTRUMENTS {
                instruments_vec[i] = inst;
            }
        }

        self.instruments = instruments_vec.try_into().expect("Wrong size");

        let sample_rate = (self.samples_per_tick as f32 * self.bpm * 4.0) / 60.0;
        self.samples_per_tick = ((sample_rate * 60.0) / (self.bpm * 4.0)) as usize;
    }
}
