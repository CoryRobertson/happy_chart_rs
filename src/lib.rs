pub(crate) mod auto_update_status;
pub(crate) mod common;
pub(crate) mod day_stats;
pub(crate) mod last_session;
pub(crate) mod options;
pub(crate) mod state;
pub(crate) mod ui;
pub(crate) mod mood_tag;

pub mod prelude {
    pub use crate::common::read_last_session_save_file;
    pub use crate::state::happy_chart_state::HappyChartState;
}

pub(crate) const SAVE_FILE_NAME: &str = "save.ser";
pub(crate) const NEW_SAVE_FILE_NAME: &str = "happy_chart_save.ser";
pub(crate) const LAST_SESSION_FILE_NAME: &str = "happy_chart_last_session.ser";
pub(crate) const BACKUP_FILENAME_PREFIX: &str = "happy_chart_backup_";
pub(crate) const MANUAL_BACKUP_SUFFIX: &str = "_manual";
pub(crate) const BACKUP_FILE_EXTENSION: &str = "zip";
pub(crate) const GIT_DESCRIBE: &str = env!("VERGEN_GIT_DESCRIBE");
pub(crate) const BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

// TODO: activity tags to add to a note, an activity tag is a selection of things that a user added to
//   a list of things they commonly do, so they can see for example, days the user went for a bike ride, or days where the user socialized
