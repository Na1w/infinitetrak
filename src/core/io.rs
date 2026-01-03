use super::instrument::Instrument;
use super::pattern::Pattern;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub bpm: f32,
    #[serde(default)]
    pub pattern: Option<Pattern>, // Legacy support
    #[serde(default)]
    pub patterns: Vec<Pattern>,
    pub instruments: Vec<Instrument>,
}

pub fn save_project(
    path: &str,
    bpm: f32,
    patterns: &[Pattern],
    instruments: &[Instrument],
) -> Result<(), Box<dyn std::error::Error>> {
    let project = Project {
        bpm,
        pattern: None,
        patterns: patterns.to_vec(),
        instruments: instruments.to_vec(),
    };

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &project)?;
    Ok(())
}

pub fn load_project(path: &str) -> Result<Project, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let project: Project = serde_json::from_reader(reader)?;
    Ok(project)
}
