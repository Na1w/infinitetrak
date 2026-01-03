use super::app::{App, InstrumentFocus};
use crate::audio::render_to_wav;
use crate::core::io::{load_project, save_project};
use crate::core::pattern::Pattern;
use crate::core::state::PlayMode;
use crate::core::{ModuleConfig, NUM_CHANNELS, NUM_INSTRUMENTS, ROWS_PER_PATTERN, WaveformType};
use crossterm::event::{self, KeyCode};
use std::fs;
use std::path::Path;

pub fn handle_file_dialog_input(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Esc => {
            app.show_file_dialog = false;
        }
        KeyCode::Up => {
            let selected = app.file_list_state.selected().unwrap_or(0);
            if selected > 0 {
                app.file_list_state.select(Some(selected - 1));
            }
        }
        KeyCode::Down => {
            let selected = app.file_list_state.selected().unwrap_or(0);
            if !app.file_list.is_empty() && selected < app.file_list.len() - 1 {
                app.file_list_state.select(Some(selected + 1));
            }
        }
        KeyCode::Enter => {
            if let Some(selected) = app.file_list_state.selected()
                && selected < app.file_list.len() {
                    let filename = app.file_list[selected].clone();
                    match load_project(&filename) {
                        Ok(project) => {
                            {
                                let mut state = app.state.lock().unwrap();
                                state.load_project(project);
                            }
                            app.set_status(format!("Loaded {}", filename));
                            app.current_filename = Some(filename);
                            app.show_file_dialog = false;
                        }
                        Err(e) => {
                            app.set_status(format!("Error loading: {}", e));
                        }
                    }
                }
        }
        _ => {}
    }
}

pub fn handle_pattern_input(key: event::KeyEvent, app: &mut App) {
    match key.code {
        // Load Project (F9) - Open File Dialog
        KeyCode::F(9) => {
            if let Ok(entries) = fs::read_dir(".") {
                app.file_list.clear();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension()
                        && ext == "json"
                            && let Some(name) = path.file_name() {
                                app.file_list.push(name.to_string_lossy().into_owned());
                            }
                }
                app.file_list.sort();
                if !app.file_list.is_empty() {
                    app.file_list_state.select(Some(0));
                    app.show_file_dialog = true;
                } else {
                    app.set_status("No .json files found".to_string());
                }
            } else {
                app.set_status("Failed to read directory".to_string());
            }
        }
        // Save New (F10)
        KeyCode::F(10) => {
            let filename = {
                let mut i = 1;
                loop {
                    let name = format!("project_{:02}.json", i);
                    if !Path::new(&name).exists() {
                        break name;
                    }
                    i += 1;
                    if i > 99 {
                        break "project_new.json".to_string();
                    }
                }
            };

            let (bpm, patterns, instruments) = {
                let state = app.state.lock().unwrap();
                (state.bpm, state.patterns.clone(), state.instruments.clone())
            };

            if let Err(e) = save_project(&filename, bpm, &patterns, &instruments) {
                app.set_status(format!("Error saving new: {}", e));
            } else {
                app.set_status(format!("Saved new to {}", filename));
                app.current_filename = Some(filename);
            }
        }
        // Save (F11) - Overwrite current or Save New if none
        KeyCode::F(11) => {
            if let Some(filename) = &app.current_filename {
                let (bpm, patterns, instruments) = {
                    let state = app.state.lock().unwrap();
                    (state.bpm, state.patterns.clone(), state.instruments.clone())
                };
                if let Err(e) = save_project(filename, bpm, &patterns, &instruments) {
                    app.set_status(format!("Error saving: {}", e));
                } else {
                    app.set_status(format!("Saved to {}", filename));
                }
            } else {
                // No current file, behave like F10 (Save New)
                let filename = {
                    let mut i = 1;
                    loop {
                        let name = format!("project_{:02}.json", i);
                        if !Path::new(&name).exists() {
                            break name;
                        }
                        i += 1;
                        if i > 99 {
                            break "project_new.json".to_string();
                        }
                    }
                };

                let (bpm, patterns, instruments) = {
                    let state = app.state.lock().unwrap();
                    (state.bpm, state.patterns.clone(), state.instruments.clone())
                };

                if let Err(e) = save_project(&filename, bpm, &patterns, &instruments) {
                    app.set_status(format!("Error saving new: {}", e));
                } else {
                    app.set_status(format!("Saved new to {}", filename));
                    app.current_filename = Some(filename);
                }
            }
        }
        // Render to WAV (F12)
        KeyCode::F(12) => {
            let state_clone = { app.state.lock().unwrap().clone() };
            app.set_status("Rendering to output.wav...".to_string());

            if let Err(e) = render_to_wav("output.wav", &state_clone) {
                app.set_status(format!("Render failed: {}", e));
            } else {
                app.set_status("Render complete: output.wav".to_string());
            }
        }

        // Navigation & Editing
        KeyCode::Down => {
            if app.cursor_row < ROWS_PER_PATTERN - 1 {
                app.cursor_row += 1;
            }
        }
        KeyCode::Up => {
            if app.cursor_row > 0 {
                app.cursor_row -= 1;
            }
        }
        KeyCode::Right => {
            if app.cursor_channel < NUM_CHANNELS - 1 {
                app.cursor_channel += 1;
                app.current_instrument_idx = app.cursor_channel;
            }
        }
        KeyCode::Left => {
            if app.cursor_channel > 0 {
                app.cursor_channel -= 1;
                app.current_instrument_idx = app.cursor_channel;
            }
        }
        KeyCode::F(1) => {
            if app.current_octave > 0 {
                app.current_octave -= 1;
            }
        }
        KeyCode::F(2) => {
            if app.current_octave < 8 {
                app.current_octave += 1;
            }
        }
        KeyCode::F(3) => {
            if app.edit_step > 0 {
                app.edit_step -= 1;
            }
        }
        KeyCode::F(4) => {
            if app.edit_step < 16 {
                app.edit_step += 1;
            }
        }
        KeyCode::F(5) => {
            let current_pattern = {
                let mut state = app.state.lock().unwrap();
                if state.current_pattern > 0 {
                    state.current_pattern -= 1;
                }
                state.current_pattern
            };
            app.set_status(format!("Pattern: {}", current_pattern));
        }
        KeyCode::F(6) => {
            let current_pattern = {
                let mut state = app.state.lock().unwrap();
                if state.current_pattern < 255 {
                    // Arbitrary limit
                    state.current_pattern += 1;
                    if state.current_pattern >= state.patterns.len() {
                        state.patterns.push(Pattern::default());
                    }
                }
                state.current_pattern
            };
            app.set_status(format!("Pattern: {}", current_pattern));
        }
        KeyCode::F(7) => {
            let mut state = app.state.lock().unwrap();
            if state.bpm > 10.0 {
                let sample_rate = (state.samples_per_tick as f32 * state.bpm * 4.0) / 60.0;
                state.bpm -= 5.0;
                state.samples_per_tick = ((sample_rate * 60.0) / (state.bpm * 4.0)) as usize;
            }
        }
        KeyCode::F(8) => {
            let mut state = app.state.lock().unwrap();
            if state.bpm < 300.0 {
                let sample_rate = (state.samples_per_tick as f32 * state.bpm * 4.0) / 60.0;
                state.bpm += 5.0;
                state.samples_per_tick = ((sample_rate * 60.0) / (state.bpm * 4.0)) as usize;
            }
        }
        KeyCode::Char('p') => {
            let play_mode = {
                let mut state = app.state.lock().unwrap();
                state.play_mode = match state.play_mode {
                    PlayMode::Pattern => PlayMode::Song,
                    PlayMode::Song => PlayMode::Pattern,
                };
                state.play_mode
            };
            app.set_status(format!("Mode: {:?}", play_mode));
        }
        KeyCode::Char('n') => {
            let (new_idx, total) = {
                let mut state = app.state.lock().unwrap();
                let current_pattern_idx = state.current_pattern;
                let current_pattern = state.patterns[current_pattern_idx].clone();
                state
                    .patterns
                    .insert(current_pattern_idx + 1, current_pattern);
                state.current_pattern += 1;
                (state.current_pattern, state.patterns.len())
            };
            app.set_status(format!("Cloned Pattern {} (Total: {})", new_idx, total));
        }
        KeyCode::Char('x') => {
            let (new_idx, total) = {
                let mut state = app.state.lock().unwrap();
                let current_pattern_idx = state.current_pattern;
                if state.patterns.len() > 1 {
                    state.patterns.remove(current_pattern_idx);
                    if state.current_pattern >= state.patterns.len() {
                        state.current_pattern = state.patterns.len() - 1;
                    }
                    (state.current_pattern, state.patterns.len())
                } else {
                    (state.current_pattern, state.patterns.len())
                }
            };
            app.set_status(format!(
                "Deleted Pattern. Current: {} (Total: {})",
                new_idx, total
            ));
        }
        KeyCode::Backspace | KeyCode::Delete | KeyCode::Char('.') => {
            let mut state = app.state.lock().unwrap();
            let pattern_idx = state.current_pattern;
            state.patterns[pattern_idx].rows[app.cursor_row][app.cursor_channel].key = 0;
            state.preview_request = Some((app.cursor_channel, 0));

            if app.edit_step > 0 {
                app.cursor_row = (app.cursor_row + app.edit_step).min(ROWS_PER_PATTERN - 1);
            }
        }
        KeyCode::Char(c) => {
            let base_note = match c {
                'z' => Some(0),
                's' => Some(1),
                'x' => Some(2),
                'd' => Some(3),
                'c' => Some(4),
                'v' => Some(5),
                'g' => Some(6),
                'b' => Some(7),
                'h' => Some(8),
                'n' => Some(9),
                'j' => Some(10),
                'm' => Some(11),
                ',' => Some(12),
                _ => None,
            };

            if let Some(base) = base_note {
                let mut state = app.state.lock().unwrap();
                let midi_note = base + (app.current_octave + 1) * 12;
                if midi_note < 128 {
                    let pattern_idx = state.current_pattern;
                    state.patterns[pattern_idx].rows[app.cursor_row][app.cursor_channel].key =
                        midi_note;
                    state.preview_request = Some((app.cursor_channel, midi_note));
                }

                if app.edit_step > 0 {
                    app.cursor_row = (app.cursor_row + app.edit_step).min(ROWS_PER_PATTERN - 1);
                }
            }
        }
        _ => {}
    }
}

pub fn handle_instrument_input(code: KeyCode, app: &mut App) {
    match app.inst_focus {
        InstrumentFocus::List => {
            match code {
                KeyCode::Up => {
                    if app.current_instrument_idx > 0 {
                        app.current_instrument_idx -= 1;
                    }
                }
                KeyCode::Down => {
                    if app.current_instrument_idx < NUM_INSTRUMENTS - 1 {
                        app.current_instrument_idx += 1;
                    }
                }
                KeyCode::Right | KeyCode::Enter => {
                    app.inst_focus = InstrumentFocus::Params;
                    app.param_idx = 0;
                    app.param_table_state.select(Some(0));
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    let digit = c.to_digit(10).unwrap() as usize;
                    let mut new_idx = (app.current_instrument_idx % 10) * 10 + digit;
                    if new_idx >= NUM_INSTRUMENTS {
                        new_idx = digit;
                    }
                    if new_idx < NUM_INSTRUMENTS {
                        app.current_instrument_idx = new_idx;
                    }
                }
                _ => {}
            }
            app.inst_list_state.select(Some(app.current_instrument_idx));
        }
        InstrumentFocus::Params => {
            match code {
                KeyCode::Up => {
                    if app.param_idx > 0 {
                        app.param_idx -= 1;
                    }
                }
                KeyCode::Down => {
                    let state = app.state.lock().unwrap();
                    let inst = &state.instruments[app.current_instrument_idx];
                    let total_params = count_params(inst);
                    if app.param_idx < total_params - 1 {
                        app.param_idx += 1;
                    }
                }
                KeyCode::Left | KeyCode::Esc => {
                    app.inst_focus = InstrumentFocus::List;
                }
                KeyCode::Char('+') => change_module_param(app, 1.0),
                KeyCode::Char('-') => change_module_param(app, -1.0),
                _ => {}
            }
            app.param_table_state.select(Some(app.param_idx));
        }
    }
}

fn count_params(inst: &crate::core::Instrument) -> usize {
    let mut count = 0;
    for module in &inst.modules {
        count += match module {
            ModuleConfig::Oscillator { .. } => 3,
            ModuleConfig::Filter { .. } => 2,
            ModuleConfig::Adsr { .. } => 4,
            ModuleConfig::Gain { .. } => 1,
        };
    }
    count
}

fn change_module_param(app: &mut App, dir: f32) {
    let mut state = app.state.lock().unwrap();
    let inst = &mut state.instruments[app.current_instrument_idx];

    let mut current_idx = 0;
    for module in &mut inst.modules {
        match module {
            ModuleConfig::Oscillator {
                waveform,
                pitch_env_amount,
                pitch_env_decay,
                ..
            } => {
                if current_idx == app.param_idx {
                    if dir > 0.0 {
                        *waveform = match waveform {
                            WaveformType::Sine => WaveformType::Square,
                            WaveformType::Square => WaveformType::Saw,
                            WaveformType::Saw => WaveformType::Triangle,
                            WaveformType::Triangle => WaveformType::Noise,
                            WaveformType::Noise => WaveformType::Sine,
                        };
                    } else {
                        *waveform = match waveform {
                            WaveformType::Sine => WaveformType::Noise,
                            WaveformType::Square => WaveformType::Sine,
                            WaveformType::Saw => WaveformType::Square,
                            WaveformType::Triangle => WaveformType::Saw,
                            WaveformType::Noise => WaveformType::Triangle,
                        };
                    }
                    return;
                }
                current_idx += 1;
                if current_idx == app.param_idx {
                    *pitch_env_amount = (*pitch_env_amount + dir * 10.0).max(0.0);
                    return;
                }
                current_idx += 1;
                if current_idx == app.param_idx {
                    *pitch_env_decay = (*pitch_env_decay + dir * 0.01).max(0.001);
                    return;
                }
                current_idx += 1;
            }
            ModuleConfig::Filter { cutoff, resonance } => {
                if current_idx == app.param_idx {
                    *cutoff = (*cutoff + dir * 100.0).clamp(20.0, 20000.0);
                    return;
                }
                current_idx += 1;
                if current_idx == app.param_idx {
                    *resonance = (*resonance + dir * 0.05).clamp(0.0, 0.95);
                    return;
                }
                current_idx += 1;
            }
            ModuleConfig::Adsr {
                attack,
                decay,
                sustain,
                release,
            } => {
                if current_idx == app.param_idx {
                    *attack = (*attack + dir * 0.01).max(0.001);
                    return;
                }
                current_idx += 1;
                if current_idx == app.param_idx {
                    *decay = (*decay + dir * 0.05).max(0.001);
                    return;
                }
                current_idx += 1;
                if current_idx == app.param_idx {
                    *sustain = (*sustain + dir * 0.05).clamp(0.0, 1.0);
                    return;
                }
                current_idx += 1;
                if current_idx == app.param_idx {
                    *release = (*release + dir * 0.05).max(0.001);
                    return;
                }
                current_idx += 1;
            }
            ModuleConfig::Gain { level } => {
                if current_idx == app.param_idx {
                    *level = (*level + dir * 0.05).clamp(0.0, 1.0);
                    return;
                }
                current_idx += 1;
            }
        }
    }
}
