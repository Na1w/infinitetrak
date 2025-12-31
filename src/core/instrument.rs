use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum WaveformType {
    Sine,
    Square,
    Saw,
    Triangle,
    Noise,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ModuleConfig {
    Oscillator {
        waveform: WaveformType,
        pitch_offset: f32,
        detune: f32,
        pitch_env_amount: f32,
        pitch_env_decay: f32,
    },
    Filter {
        cutoff: f32,
        resonance: f32,
    },
    Adsr {
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    },
    Gain {
        level: f32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Instrument {
    pub name: String,
    pub modules: Vec<ModuleConfig>,
}

impl Default for Instrument {
    fn default() -> Self {
        Self {
            name: "Init".to_string(),
            modules: vec![
                ModuleConfig::Oscillator {
                    waveform: WaveformType::Sine,
                    pitch_offset: 0.0,
                    detune: 0.0,
                    pitch_env_amount: 0.0,
                    pitch_env_decay: 0.1
                },
                ModuleConfig::Adsr { attack: 0.01, decay: 0.1, sustain: 0.8, release: 0.2 },
                ModuleConfig::Gain { level: 0.5 },
            ],
        }
    }
}
