use infinitedsp_core::core::audio_param::AudioParam;
use infinitedsp_core::core::channels::Mono;
use infinitedsp_core::core::dsp_chain::DspChain;
use infinitedsp_core::core::frame_processor::FrameProcessor;
use infinitedsp_core::core::parameter::Parameter;
use infinitedsp_core::effects::filter::ladder_filter::LadderFilter;
use infinitedsp_core::effects::utility::dc_source::DcSource;
use infinitedsp_core::effects::utility::gain::Gain;
use infinitedsp_core::synthesis::envelope::{Adsr, Trigger};
use infinitedsp_core::synthesis::oscillator::{Oscillator, Waveform};

use crate::core::{Instrument, ModuleConfig, WaveformType};

struct RuntimeModule {
    processor: Box<dyn FrameProcessor<Mono> + Send>,
    params: Vec<Parameter>,
    config_type: ModuleConfig,
}

pub struct SynthVoice {
    modules: Vec<RuntimeModule>,
    triggers: Vec<Trigger>,
    pitch: Parameter,
    gate: Parameter,
    sample_rate: f32,
}

impl SynthVoice {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            modules: Vec::new(),
            triggers: Vec::new(),
            pitch: Parameter::new(440.0),
            gate: Parameter::new(0.0),
            sample_rate,
        }
    }

    pub fn build(&mut self, instrument: &Instrument) {
        self.modules.clear();
        self.triggers.clear();

        for config in &instrument.modules {
            match config {
                ModuleConfig::Oscillator {
                    waveform,
                    pitch_offset: _,
                    detune: _,
                    pitch_env_amount,
                    pitch_env_decay,
                } => {
                    let wave = match waveform {
                        WaveformType::Sine => Waveform::Sine,
                        WaveformType::Square => Waveform::Square,
                        WaveformType::Saw => Waveform::Saw,
                        WaveformType::Triangle => Waveform::Triangle,
                        WaveformType::Noise => Waveform::WhiteNoise,
                    };

                    let mut osc = if *pitch_env_amount > 0.0 {
                        let p_pe_amount = Parameter::new(*pitch_env_amount);
                        let p_pe_decay = Parameter::new(*pitch_env_decay);

                        let base_pitch = DcSource::new(AudioParam::Linked(self.pitch.clone()));

                        let pe_env = Adsr::new(
                            AudioParam::Linked(self.gate.clone()),
                            AudioParam::Static(0.001),
                            AudioParam::Linked(p_pe_decay.clone()),
                            AudioParam::Static(0.0),
                            AudioParam::Static(0.01),
                        );
                        self.triggers.push(pe_env.create_trigger());

                        let pe_gain = Gain::new(AudioParam::Linked(p_pe_amount.clone()));

                        let env_chain = DspChain::new(pe_env, self.sample_rate).and(pe_gain);

                        let freq_mod =
                            DspChain::new(base_pitch, self.sample_rate).and_mix(1.0, env_chain);

                        let osc = Oscillator::new(AudioParam::Dynamic(Box::new(freq_mod)), wave);

                        let mut osc_box = Box::new(osc);
                        osc_box.set_sample_rate(self.sample_rate);

                        self.modules.push(RuntimeModule {
                            processor: osc_box,
                            params: vec![p_pe_amount, p_pe_decay],
                            config_type: config.clone(),
                        });
                        continue;
                    } else {
                        Oscillator::new(AudioParam::Linked(self.pitch.clone()), wave)
                    };

                    osc.set_sample_rate(self.sample_rate);

                    self.modules.push(RuntimeModule {
                        processor: Box::new(osc),
                        params: vec![],
                        config_type: config.clone(),
                    });
                }
                ModuleConfig::Filter { cutoff, resonance } => {
                    let p_cutoff = Parameter::new(*cutoff);
                    let p_res = Parameter::new(*resonance);

                    let mut filter = LadderFilter::new(
                        AudioParam::Linked(p_cutoff.clone()),
                        AudioParam::Linked(p_res.clone()),
                    );
                    filter.set_sample_rate(self.sample_rate);

                    self.modules.push(RuntimeModule {
                        processor: Box::new(filter),
                        params: vec![p_cutoff, p_res],
                        config_type: config.clone(),
                    });
                }
                ModuleConfig::Adsr {
                    attack,
                    decay,
                    sustain,
                    release,
                } => {
                    let p_a = Parameter::new(*attack);
                    let p_d = Parameter::new(*decay);
                    let p_s = Parameter::new(*sustain);
                    let p_r = Parameter::new(*release);

                    let env = Adsr::new(
                        AudioParam::Linked(self.gate.clone()),
                        AudioParam::Linked(p_a.clone()),
                        AudioParam::Linked(p_d.clone()),
                        AudioParam::Linked(p_s.clone()),
                        AudioParam::Linked(p_r.clone()),
                    );
                    self.triggers.push(env.create_trigger());

                    let mut vca = Gain::new(AudioParam::Dynamic(Box::new(env)));
                    FrameProcessor::<Mono>::set_sample_rate(&mut vca, self.sample_rate);

                    self.modules.push(RuntimeModule {
                        processor: Box::new(vca),
                        params: vec![p_a, p_d, p_s, p_r],
                        config_type: config.clone(),
                    });
                }
                ModuleConfig::Gain { level } => {
                    let p_level = Parameter::new(*level);
                    let mut gain = Gain::new(AudioParam::Linked(p_level.clone()));
                    FrameProcessor::<Mono>::set_sample_rate(&mut gain, self.sample_rate);

                    self.modules.push(RuntimeModule {
                        processor: Box::new(gain),
                        params: vec![p_level],
                        config_type: config.clone(),
                    });
                }
            }
        }
    }

    pub fn update_params(&mut self, freq: f32, instrument: Option<&Instrument>) {
        self.pitch.set(freq);

        if let Some(inst) = instrument {
            let needs_rebuild = if self.modules.len() != inst.modules.len() {
                true
            } else {
                self.modules.iter().zip(&inst.modules).any(|(rt, cfg)| {
                    match (&rt.config_type, cfg) {
                        (
                            ModuleConfig::Oscillator {
                                waveform: w1,
                                pitch_env_amount: a1,
                                ..
                            },
                            ModuleConfig::Oscillator {
                                waveform: w2,
                                pitch_env_amount: a2,
                                ..
                            },
                        ) => w1 != w2 || (*a1 > 0.0) != (*a2 > 0.0),
                        (ModuleConfig::Filter { .. }, ModuleConfig::Filter { .. }) => false,
                        (ModuleConfig::Adsr { .. }, ModuleConfig::Adsr { .. }) => false,
                        (ModuleConfig::Gain { .. }, ModuleConfig::Gain { .. }) => false,
                        _ => true,
                    }
                })
            };

            if needs_rebuild {
                self.build(inst);
            } else {
                for (rt, cfg) in self.modules.iter_mut().zip(&inst.modules) {
                    match (cfg, &rt.params) {
                        (
                            ModuleConfig::Oscillator {
                                pitch_env_amount,
                                pitch_env_decay,
                                ..
                            },
                            params,
                        ) => {
                            if !params.is_empty() {
                                params[0].set(*pitch_env_amount);
                                params[1].set(*pitch_env_decay);
                            }
                        }
                        (ModuleConfig::Filter { cutoff, resonance }, params) => {
                            if params.len() >= 2 {
                                params[0].set(*cutoff);
                                params[1].set(*resonance);
                            }
                        }
                        (
                            ModuleConfig::Adsr {
                                attack,
                                decay,
                                sustain,
                                release,
                            },
                            params,
                        ) => {
                            if params.len() >= 4 {
                                params[0].set(*attack);
                                params[1].set(*decay);
                                params[2].set(*sustain);
                                params[3].set(*release);
                            }
                        }
                        (ModuleConfig::Gain { level }, params) => {
                            if !params.is_empty() {
                                params[0].set(*level);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn trigger(&mut self) {
        self.gate.set(1.0);
        for trigger in &self.triggers {
            trigger.fire();
        }
    }

    pub fn release(&mut self) {
        self.gate.set(0.0);
    }
}

impl FrameProcessor<Mono> for SynthVoice {
    fn process(&mut self, buffer: &mut [f32], sample_index: u64) {
        for module in &mut self.modules {
            module.processor.process(buffer, sample_index);
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        for module in &mut self.modules {
            module.processor.set_sample_rate(sample_rate);
        }
    }

    fn latency_samples(&self) -> u32 {
        self.modules
            .iter()
            .map(|m| m.processor.latency_samples())
            .sum()
    }

    fn name(&self) -> &str {
        "SynthVoice"
    }

    fn visualize(&self, indent: usize) -> String {
        let mut output = String::new();
        for module in &self.modules {
            output.push_str(&module.processor.visualize(indent));
            output.push('\n');
        }
        output
    }
}
