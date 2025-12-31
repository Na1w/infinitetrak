pub mod instrument;
pub mod pattern;
pub mod state;
pub mod io;

pub use instrument::{Instrument, ModuleConfig, WaveformType};
pub use pattern::{ROWS_PER_PATTERN, NUM_CHANNELS};
pub use state::SharedState;

pub const NUM_INSTRUMENTS: usize = 32;
