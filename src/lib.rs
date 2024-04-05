#![deny(clippy::suboptimal_flops, clippy::cast_lossless)]
#![allow(clippy::uninlined_format_args)]
pub(crate) mod common;
pub(crate) mod day_stats;
pub(crate) mod options;
pub(crate) mod state;
pub(crate) mod ui;

pub mod prelude {
    pub use crate::common::mood_tag::*;
    pub use crate::common::save::read_last_session_save_file;
    pub use crate::common::save::read_save_file;
    pub use crate::day_stats::improved_daystat::*;
    pub use crate::state::happy_chart_state::HappyChartState;
}

// TODO: use source engine / half-life menu sounds when clicking and mousing over buttons

pub(crate) const SAVE_FILE_NAME: &str = "save.ser";
pub(crate) const NEW_SAVE_FILE_NAME: &str = "happy_chart_save.ser";
pub(crate) const LAST_SESSION_FILE_NAME: &str = "happy_chart_last_session.ser";
pub(crate) const BACKUP_FILENAME_PREFIX: &str = "happy_chart_backup_";
pub(crate) const MANUAL_BACKUP_SUFFIX: &str = "_manual";
pub(crate) const BACKUP_FILE_EXTENSION: &str = "zip";
pub(crate) const GIT_DESCRIBE: &str = env!("VERGEN_GIT_DESCRIBE");
pub(crate) const BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");
pub(crate) const MIN_ENCRYPT_KEY_LENGTH: usize = 4;
pub(crate) const MAX_ENCRYPT_KEY_LENGTH: usize = 32;
pub(crate) const NOTE_OLD_NUM_DAYS: u32 = 3;
pub(crate) const LOG_FILE_NAME: &str = "happy_chart_rs.log";
