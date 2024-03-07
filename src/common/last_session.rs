use crate::options::program_options::ProgramOptions;
use crate::state::tutorial_state::TutorialGoal;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct LastSession {
    pub window_size: [f32; 2],

    pub program_options: ProgramOptions,

    pub open_modulus: i32,

    pub last_open_date: DateTime<Local>,

    pub last_version_checked: Option<String>,

    pub last_backup_date: DateTime<Local>,

    pub tutorial_state: TutorialGoal,
}

impl Default for LastSession {
    #[tracing::instrument]
    fn default() -> Self {
        Self {
            window_size: [800.0, 600.0],
            program_options: ProgramOptions::default(),
            open_modulus: 0,
            last_open_date: Local::now(),
            last_version_checked: None,
            last_backup_date: Local::now(),
            tutorial_state: TutorialGoal::default(),
        }
    }
}
