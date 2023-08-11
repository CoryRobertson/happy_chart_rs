use crate::program_options::ProgramOptions;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LastSession {
    #[serde(default)]
    pub window_size: [f32; 2],
    #[serde(default)]
    pub program_options: ProgramOptions,
}

impl Default for LastSession {
    fn default() -> Self {
        Self {
            window_size: [800.0, 600.0],
            program_options: ProgramOptions::default(),
        }
    }
}
