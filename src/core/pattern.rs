use serde::{Serialize, Deserialize};

pub const NUM_CHANNELS: usize = 16;
pub const ROWS_PER_PATTERN: usize = 64;

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct Note {
    pub key: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Pattern {
    // Use Vec for rows to avoid serde array limit issues.
    // Inner array [Note; 16] is supported by serde (<= 32).
    pub rows: Vec<[Note; NUM_CHANNELS]>,
}

impl Default for Pattern {
    fn default() -> Self {
        let mut rows = Vec::with_capacity(ROWS_PER_PATTERN);
        for _ in 0..ROWS_PER_PATTERN {
            rows.push([Note::default(); NUM_CHANNELS]);
        }
        Self {
            rows,
        }
    }
}
