use crate::program_options::ProgramOptions;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LastSession {
    #[serde(default)]
    pub window_size: [f32; 2],
    #[serde(default)]
    pub program_options: ProgramOptions,
    #[serde(default)]
    pub open_modulus: i32,
    #[serde(default)]
    pub last_open_date: DateTime<Local>,
    #[serde(default)]
    pub last_version_checked: Option<String>,
    #[serde(default)]
    pub last_backup_date: DateTime<Local>,
}

impl Default for LastSession {
    fn default() -> Self {
        Self {
            window_size: [800.0, 600.0],
            program_options: ProgramOptions::default(),
            open_modulus: 0,
            last_open_date: Local::now(),
            last_version_checked: None,
            last_backup_date: Local::now(),
        }
    }
}
