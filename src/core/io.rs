use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use super::pattern::Pattern;
use super::instrument::Instrument;

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub bpm: f32,
    pub pattern: Pattern,
    pub instruments: Vec<Instrument>,
}

pub fn save_project(path: &str, bpm: f32, pattern: &Pattern, instruments: &[Instrument]) -> Result<(), Box<dyn std::error::Error>> {
    let project = Project {
        bpm,
        pattern: pattern.clone(),
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
