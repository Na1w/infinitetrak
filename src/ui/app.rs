use std::io;
use std::sync::{Arc, Mutex};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::{ListState, TableState},
    Terminal,
};
use crate::core::SharedState;
use super::view::ui;
use super::input::{handle_pattern_input, handle_instrument_input, handle_file_dialog_input};

#[derive(PartialEq)]
pub enum View {
    Pattern,
    Instrument,
}

#[derive(PartialEq)]
pub enum InstrumentFocus {
    List,
    Params,
}

pub struct App {
    pub state: Arc<Mutex<SharedState>>,
    pub cursor_row: usize,
    pub cursor_channel: usize,
    pub current_octave: u8,
    pub current_view: View,
    pub current_instrument_idx: usize,

    pub inst_focus: InstrumentFocus,
    pub param_idx: usize,

    pub edit_step: usize,

    // UI States for scrolling
    pub inst_list_state: ListState,
    pub param_table_state: TableState,

    // Status message
    pub status_message: String,
    pub status_timer: usize,

    // File Dialog
    pub show_file_dialog: bool,
    pub file_list: Vec<String>,
    pub file_list_state: ListState,

    // Help Dialog
    pub show_help_dialog: bool,
    pub help_list_state: ListState,

    // Current Project File
    pub current_filename: Option<String>,
}

impl App {
    pub fn new(state: Arc<Mutex<SharedState>>) -> App {
        let mut inst_list_state = ListState::default();
        inst_list_state.select(Some(0));

        let mut param_table_state = TableState::default();
        param_table_state.select(Some(0));

        let file_list_state = ListState::default();
        let help_list_state = ListState::default();

        App {
            state,
            cursor_row: 0,
            cursor_channel: 0,
            current_octave: 4,
            current_view: View::Pattern,
            current_instrument_idx: 0,
            inst_focus: InstrumentFocus::List,
            param_idx: 0,
            edit_step: 1,
            inst_list_state,
            param_table_state,
            status_message: String::from("Welcome to InfiniTrak! Press ? for help."),
            status_timer: 100,
            show_file_dialog: false,
            file_list: Vec::new(),
            file_list_state,
            show_help_dialog: false,
            help_list_state,
            current_filename: None,
        }
    }

    pub fn set_status(&mut self, msg: String) {
        self.status_message = msg;
        self.status_timer = 100;
    }
}

pub fn run_app(mut app: App) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    // Removed EnableMouseCapture
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_main_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    // Removed DisableMouseCapture
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_main_loop<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> where <B as Backend>::Error: Send + Sync + 'static {
    loop {
        if let Err(e) = terminal.draw(|f| ui(f, app)) {
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }

        if app.status_timer > 0 {
            app.status_timer -= 1;
            if app.status_timer == 0 {
                app.status_message.clear();
            }
        }

        if event::poll(std::time::Duration::from_millis(50))? {
            // Only process Key events, ignore Mouse events
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if app.show_help_dialog {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('?') => {
                            app.show_help_dialog = false;
                        }
                        KeyCode::Up => {
                            let selected = app.help_list_state.selected().unwrap_or(0);
                            if selected > 0 {
                                app.help_list_state.select(Some(selected - 1));
                            }
                        }
                        KeyCode::Down => {
                            let selected = app.help_list_state.selected().unwrap_or(0);
                            // We don't know the exact length here easily without duplicating the list,
                            // but we can just let it scroll. The view will clamp it or we can set a high limit.
                            // A better way is to store the help text in App or a constant.
                            // For now, let's just increment.
                            app.help_list_state.select(Some(selected + 1));
                        }
                        _ => {}
                    }
                    continue;
                }

                if app.show_file_dialog {
                    handle_file_dialog_input(key.code, app);
                    continue;
                }

                match key.code {
                    KeyCode::Char('?') => {
                        app.show_help_dialog = true;
                        app.help_list_state.select(Some(0)); // Reset scroll
                        continue;
                    }
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Tab => {
                        app.current_view = match app.current_view {
                            View::Pattern => View::Instrument,
                            View::Instrument => View::Pattern,
                        };
                    }
                    KeyCode::Char(' ') => {
                        let mut state = app.state.lock().unwrap();
                        if !state.is_playing {
                            if key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
                                state.current_row = app.cursor_row;
                            } else {
                                state.current_row = 0;
                            }
                            state.current_tick_samples = state.samples_per_tick;
                            state.is_playing = true;
                        } else {
                            state.is_playing = false;
                        }
                    }
                    KeyCode::Enter if app.current_view == View::Pattern => {
                        let mut state = app.state.lock().unwrap();
                        if !state.is_playing {
                            state.current_row = app.cursor_row;
                            state.current_tick_samples = state.samples_per_tick;
                            state.is_playing = true;
                        } else {
                            state.is_playing = false;
                        }
                    }
                    _ => {}
                }

                match app.current_view {
                    View::Pattern => handle_pattern_input(key, app),
                    View::Instrument => handle_instrument_input(key.code, app),
                }
            }
        }
    }
}
