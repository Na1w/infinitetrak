pub mod instrument;
pub mod io;
pub mod pattern;
pub mod state;

pub use instrument::{Instrument, ModuleConfig, WaveformType};
pub use pattern::{NUM_CHANNELS, ROWS_PER_PATTERN};
pub use state::SharedState;

pub const NUM_INSTRUMENTS: usize = 32;
