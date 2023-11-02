use crate::auto_update_status::AutoUpdateStatus;
use crate::improved_daystat::ImprovedDayStat;
use crate::program_options::ProgramOptions;
use crate::BACKUP_FILENAME_PREFIX;
use chrono::{DateTime, Local};
use self_update::update::Release;
use self_update::Status;
use std::cell::Cell;
use std::fs;
use std::fs::DirEntry;
use std::thread::JoinHandle;

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

    /// The last date that a backup was taken
    pub last_backup_date: DateTime<Local>,

    /// A string of text to search through all day stats to check if they contain this string, the stats are highlighted when they contain it
    pub filter_term: String,

    pub showing_about_page: bool,
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
            last_backup_date: Local::now(),
            filter_term: "".to_string(),
            showing_about_page: false,
        }
    }

    pub fn remove_old_backup_files(&self) {
        let list = self.get_backup_file_list();

        // only remove old backup files if the number of old backups exceeds the amount allowed
        let mut removed_count = 0;
        let number_to_remove = list.len() as i32 - self.program_options.number_of_kept_backups;
        if list.len() > self.program_options.number_of_kept_backups as usize && self.program_options.number_of_kept_backups >= 0 {
            for entry in list {
                if removed_count >= number_to_remove { break; }
                let res = fs::remove_file(entry.path());
                removed_count += 1;
                println!("Removing {:?}, result: {:?}", entry, res);
            }
        }
    }

    pub fn get_backup_file_list(&self) -> Vec<DirEntry> {
        if self.program_options.backup_age_keep_days < 0 {
            return vec![];
        }

        #[cfg(debug_assertions)]
        println!("{:?}", self.program_options.backup_save_path);
        match fs::read_dir(&self.program_options.backup_save_path) {
            Ok(dir_list) => {
                dir_list
                    .filter_map(|item| {
                        #[cfg(debug_assertions)]
                        println!("{:?}", item);
                        item.ok()
                    })
                    .filter(|entry| {
                        let mut keep = false;
                        if let Some(f_name) = entry.file_name().to_str() {
                            if f_name.contains(BACKUP_FILENAME_PREFIX) && f_name.contains(".zip") {
                                if let Ok(meta_data) = entry.metadata() {
                                    if let Ok(created_time) = meta_data.created() {
                                        let dt: DateTime<Local> = created_time.into();
                                        let days =
                                            Local::now().signed_duration_since(dt).num_days();
                                        let hours =
                                            Local::now().signed_duration_since(dt).num_hours();
                                        #[cfg(debug_assertions)]
                                        {
                                            println!(
                                                "{} age: {} days hours: {}",
                                                f_name, days, hours
                                            );
                                        }
                                        keep = days > self.program_options.backup_age_keep_days as i64;
                                    }
                                }
                            }
                        }
                        keep
                    })
                    .collect::<Vec<DirEntry>>()
            }
            Err(_) => {
                vec![]
            }
        }
    }
}
