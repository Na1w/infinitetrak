use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, List, ListItem, Paragraph, Clear},
    text::{Span, Line},
    Frame,
};
use crate::core::{SharedState, ROWS_PER_PATTERN, ModuleConfig, NUM_CHANNELS};
use super::app::{App, View, InstrumentFocus};

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main View
                Constraint::Length(1), // Status Bar
            ]
            .as_ref(),
        )
        .split(f.size());

    let state_arc = app.state.clone();
    let state = state_arc.lock().unwrap();

    // Header
    let status = if state.is_playing { "PLAYING" } else { "STOPPED" };
    let view_str = match app.current_view {
        View::Pattern => "PATTERN",
        View::Instrument => "INSTRUMENT",
    };

    let inst_text = format!("{:02X}", app.current_instrument_idx);
    let step_text = format!("{}", app.edit_step);

    let header_spans = Line::from(vec![
        Span::raw(format!("InfiniTrak | BPM: {} | Octave: {} | Inst: ", state.bpm, app.current_octave)),
        Span::styled(inst_text, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" | Step: "),
        Span::styled(step_text, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(format!(" | View: {} | Status: {} | Tab: Switch View", view_str, status)),
    ]);

    let header_block = Block::default().borders(Borders::ALL).title(header_spans);
    f.render_widget(header_block, chunks[0]);

    // Main View
    match app.current_view {
        View::Pattern => draw_pattern_view(f, chunks[1], &state, app),
        View::Instrument => draw_instrument_view(f, chunks[1], &state, app),
    }

    // Status Bar
    let status_bar = Paragraph::new(app.status_message.as_str());
    f.render_widget(status_bar, chunks[2]);

    // File Dialog (Overlay)
    if app.show_file_dialog {
        draw_file_dialog(f, app);
    }
}

fn draw_file_dialog(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 50, f.size());
    f.render_widget(Clear, area); // Clear the background

    let items: Vec<ListItem> = app.file_list.iter()
        .map(|s| ListItem::new(s.as_str()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Load Project (Enter to load, Esc to cancel)"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(list, area, &mut app.file_list_state);
}

/// Helper to create a centered rectangle.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}


fn draw_pattern_view(f: &mut Frame, area: Rect, state: &SharedState, app: &App) {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default();
    let play_pos_style = Style::default().bg(Color::DarkGray);
    let cursor_play_style = Style::default().bg(Color::DarkGray).add_modifier(Modifier::REVERSED);

    let mut header_cells = Vec::with_capacity(NUM_CHANNELS + 1);
    header_cells.push(Cell::from("Row").style(Style::default().fg(Color::Yellow)));
    for i in 0..NUM_CHANNELS {
        header_cells.push(Cell::from(format!("{:X}", i)).style(Style::default().fg(Color::Yellow)));
    }

    let header = Row::new(header_cells).style(normal_style).height(1).bottom_margin(1);

    let inner_height = (area.height as usize).saturating_sub(4);

    let center_row = if state.is_playing { state.current_row } else { app.cursor_row };

    let half_height = inner_height / 2;
    let start_row = center_row.saturating_sub(half_height);

    let start_row = if start_row + inner_height > ROWS_PER_PATTERN {
        ROWS_PER_PATTERN.saturating_sub(inner_height)
    } else {
        start_row
    };

    let rows = state.pattern.rows.iter().enumerate()
        .skip(start_row)
        .take(inner_height)
        .map(|(i, row_data)| {
        let mut cells = Vec::with_capacity(NUM_CHANNELS + 1);
        cells.push(Cell::from(format!("{:02X}", i)));

        for (ch_idx, note) in row_data.iter().enumerate() {
            let note_str = if note.key == 0 {
                "---".to_string()
            } else {
                let notes = ["C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-"];
                let octave = (note.key / 12) as i8 - 1;
                let note_idx = (note.key % 12) as usize;
                format!("{}{}", notes[note_idx], octave)
            };

            let cell = Cell::from(note_str);

            if i == app.cursor_row && ch_idx == app.cursor_channel {
                if i == state.current_row {
                     cells.push(cell.style(cursor_play_style));
                } else {
                     cells.push(cell.style(selected_style));
                }
            } else {
                cells.push(cell);
            }
        }

        let row_style = if i == state.current_row {
            play_pos_style
        } else {
            normal_style
        };

        Row::new(cells).style(row_style)
    });

    let mut widths = Vec::with_capacity(NUM_CHANNELS + 1);
    widths.push(Constraint::Length(4));
    for _ in 0..NUM_CHANNELS {
        widths.push(Constraint::Length(4));
    }

    let t = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Pattern 0"));

    f.render_widget(t, area);
}

fn draw_instrument_view(f: &mut Frame, area: Rect, state: &SharedState, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(area);

    // Left: Instrument List
    let items: Vec<ListItem> = state.instruments.iter().enumerate().map(|(i, inst)| {
        let style = if i == app.current_instrument_idx {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        ListItem::new(format!("{:02X} - {}", i, inst.name)).style(style)
    }).collect();

    let list_block = Block::default().borders(Borders::ALL).title("Instruments");
    let list = List::new(items).block(list_block).highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(list, chunks[0], &mut app.inst_list_state);

    // Right: Parameters (Flattened list of all module params)
    let inst = &state.instruments[app.current_instrument_idx];

    let mut params = Vec::new();
    for (mod_idx, module) in inst.modules.iter().enumerate() {
        let prefix = format!("M{}: ", mod_idx);
        match module {
            ModuleConfig::Oscillator { waveform, pitch_env_amount, pitch_env_decay, .. } => {
                params.push((format!("{}Osc Wave", prefix), format!("{:?}", waveform)));
                params.push((format!("{}PE Amt", prefix), format!("{:.0} Hz", pitch_env_amount)));
                params.push((format!("{}PE Dec", prefix), format!("{:.3} s", pitch_env_decay)));
            },
            ModuleConfig::Filter { cutoff, resonance } => {
                params.push((format!("{}Filt Cut", prefix), format!("{:.0} Hz", cutoff)));
                params.push((format!("{}Filt Res", prefix), format!("{:.2}", resonance)));
            },
            ModuleConfig::Adsr { attack, decay, sustain, release } => {
                params.push((format!("{}Env Att", prefix), format!("{:.3} s", attack)));
                params.push((format!("{}Env Dec", prefix), format!("{:.3} s", decay)));
                params.push((format!("{}Env Sus", prefix), format!("{:.2}", sustain)));
                params.push((format!("{}Env Rel", prefix), format!("{:.3} s", release)));
            },
            ModuleConfig::Gain { level } => {
                params.push((format!("{}Gain Lvl", prefix), format!("{:.2}", level)));
            }
        }
    }

    let rows = params.iter().enumerate().map(|(i, (name, val))| {
        let style = if app.inst_focus == InstrumentFocus::Params && i == app.param_idx {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        Row::new(vec![Cell::from(name.as_str()), Cell::from(val.as_str())]).style(style)
    });

    let param_block = Block::default().borders(Borders::ALL).title("Modules & Params");
    let table = Table::new(rows, [Constraint::Length(20), Constraint::Length(15)])
        .block(param_block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(table, chunks[1], &mut app.param_table_state);
}
