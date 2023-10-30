use crate::auto_update_status::AutoUpdateStatus;
use crate::improved_daystat::ImprovedDayStat;
use crate::program_options::ProgramOptions;
use self_update::Status;
use std::cell::Cell;
use std::thread::JoinHandle;
use chrono::{DateTime, Local};
use self_update::update::Release;

#[derive(Default)]
pub struct HappyChartState {
    pub rating: f64,
    pub days: Vec<ImprovedDayStat>,
    pub first_load: bool,
    pub note_input: String,
    /// The length of days recorded since the last session. Used to determine if the user has made changes to the day list
    pub starting_length: usize,
    pub showing_options_menu: bool,
    pub program_options: ProgramOptions,
    /// The status on updating the program, see the enum for more information
    pub update_status: AutoUpdateStatus,
    pub update_thread: Cell<Option<JoinHandle<Result<Status, String>>>>,
    pub open_modulus: i32,
    /// The date and time the user last opened the program, used for determining if we should even check for an update
    pub last_open_date: DateTime<Local>,
    /// The release that is newer than the current release the user is running.
    pub update_available: Option<Release>,
    /// The version number of the most recent available update that the user has seen
    /// This variable will determine if an update message should be shown, if they have already seen the message, and ignored it then we will not tell them again.
    pub auto_update_seen_version: Option<String>,

    pub backup_path_text: String,

    pub last_backup_date: DateTime<Local>,
}

impl HappyChartState {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            rating: 0.0,
            days: vec![],
            first_load: true,
            note_input: "".to_string(),
            starting_length: 0,
            showing_options_menu: false,
            program_options: ProgramOptions::default(),
            update_status: AutoUpdateStatus::NotChecked,
            update_thread: Cell::new(None),
            open_modulus: 0,
            last_open_date: Local::now(),
            update_available: None,
            auto_update_seen_version: None,
            backup_path_text: "".into(),
            last_backup_date: Local::now(),
        }
    }
}
