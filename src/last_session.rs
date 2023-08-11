use crate::program_options::ProgramOptions;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LastSession {
    pub graph_xscale: f32,
    pub graph_yscale: f32,
    pub xoffset: i32,
    pub displaying_day_lines: bool,
    #[serde(default)]
    pub window_size: [f32; 2],
    #[serde(default)]
    pub program_options: ProgramOptions,
}

impl Default for LastSession {
    fn default() -> Self {
        LastSession {
            graph_xscale: 1.0,
            graph_yscale: 1.0,
            xoffset: 0,
            displaying_day_lines: false,
            window_size: [800.0, 600.0],
            program_options: ProgramOptions::default(),
        }
    }
}
