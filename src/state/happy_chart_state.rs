use crate::auto_update_status::AutoUpdateStatus;
use crate::day_stats::improved_daystat::ImprovedDayStat;
use crate::mood_tag::MoodTag;
use crate::options::program_options::ProgramOptions;
use crate::state::error_states::HappyChartError;
use crate::state::state_stats::StateStats;
use crate::state::tutorial_state::TutorialGoal;
use crate::{BACKUP_FILENAME_PREFIX, BACKUP_FILE_EXTENSION, MANUAL_BACKUP_SUFFIX};
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

    pub stats: StateStats,

    /// List of error states that are present, we show the user every item in this list if any exist
    pub error_states: Vec<HappyChartError>,

    pub showing_mood_tag_selector: bool,
    pub mood_selection_list: Vec<MoodTag>,

    pub showing_statistics_screen: bool,

    /// The position of the day lines offset to be calculated from
    pub central_screen_ui_delta_pos: Option<UiDelta>,

    pub tutorial_state: TutorialGoal,
}

#[derive(Debug, Clone)]
pub struct UiDelta {
    starting_amount: f32,
    current_amount: f32,
}

impl UiDelta {
    pub const fn new(starting: f32) -> Self {
        Self {
            starting_amount: starting,
            current_amount: starting,
        }
    }

    #[tracing::instrument]
    pub fn get_delta(&self) -> f32 {
        self.current_amount - self.starting_amount
    }

    #[tracing::instrument]
    pub fn update_current(&mut self, new_amount: f32) {
        self.current_amount = new_amount;
    }
}

impl Default for UiDelta {
    fn default() -> Self {
        Self {
            starting_amount: 155.5,
            current_amount: 155.5,
        }
    }
}

impl HappyChartState {
    /// Magic number that makes day lines look just right
    const DAY_LINE_OFFSET: f32 = 165.0;

    #[tracing::instrument(skip(_cc))]
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            rating: 0.0,
            days: vec![],
            first_load: true,
            note_input: String::new(),
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
            filter_term: String::new(),
            showing_about_page: false,
            stats: StateStats::new(),
            error_states: vec![],
            showing_mood_tag_selector: false,
            mood_selection_list: vec![],
            showing_statistics_screen: false,
            central_screen_ui_delta_pos: None,
            tutorial_state: Default::default(),
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn remove_old_backup_files(&self) {
        let list = self.get_backup_file_list();

        // only remove old backup files if the number of old backups exceeds the amount allowed
        let mut removed_count = 0;
        let number_to_remove = list.len() as i32 - self.program_options.number_of_kept_backups;
        if list.len() > self.program_options.number_of_kept_backups as usize
            && self.program_options.number_of_kept_backups >= 0
        {
            for entry in list {
                if removed_count >= number_to_remove {
                    break;
                }
                let res = fs::remove_file(entry.path());
                removed_count += 1;
                println!("Removing {:?}, result: {:?}", entry, res);
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn get_backup_file_list(&self) -> Vec<DirEntry> {
        if self.program_options.backup_age_keep_days < 0 {
            return vec![];
        }

        #[cfg(debug_assertions)]
        println!("{:?}", self.program_options.backup_save_path);
        match fs::read_dir(&self.program_options.backup_save_path) {
            Ok(dir_list) => dir_list
                .filter_map(|item| {
                    #[cfg(debug_assertions)]
                    println!("{:?}", item);
                    item.ok()
                })
                .filter(|entry| {
                    let mut keep = false;
                    if let Some(f_name) = entry.file_name().to_str() {
                        if !f_name.contains(MANUAL_BACKUP_SUFFIX)
                            && f_name.contains(BACKUP_FILENAME_PREFIX)
                            && f_name.contains(BACKUP_FILE_EXTENSION)
                        {
                            if let Ok(meta_data) = entry.metadata() {
                                if let Ok(created_time) = meta_data.created() {
                                    let dt: DateTime<Local> = created_time.into();
                                    let days = Local::now().signed_duration_since(dt).num_days();

                                    #[cfg(debug_assertions)]
                                    {
                                        let hours =
                                            Local::now().signed_duration_since(dt).num_hours();
                                        println!("{} age: {} days hours: {}", f_name, days, hours);
                                    }
                                    keep =
                                        days > i64::from(self.program_options.backup_age_keep_days);
                                }
                            }
                        }
                    }
                    keep
                })
                .collect::<Vec<DirEntry>>(),
            Err(_) => {
                vec![]
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn get_day_line_y_value(&self) -> f32 {
        Self::DAY_LINE_OFFSET - self.program_options.day_line_height_offset + {
            if self.program_options.move_day_lines_with_ui {
                self.central_screen_ui_delta_pos
                    .as_ref()
                    .map_or(0.0, |ui_delta| ui_delta.get_delta())
            } else {
                0.0
            } // use 0 as an offset if the user does not want the day lines to move with the ui
        }
    }
}
