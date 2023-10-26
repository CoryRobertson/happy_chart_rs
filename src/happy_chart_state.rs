use crate::auto_update_status::AutoUpdateStatus;
use crate::improved_daystat::ImprovedDayStat;
use crate::program_options::ProgramOptions;
use self_update::Status;
use std::cell::Cell;
use std::thread::JoinHandle;

#[derive(Default)]
pub struct HappyChartState {
    pub rating: f64,
    pub days: Vec<ImprovedDayStat>,
    pub first_load: bool,
    pub note_input: String,
    pub starting_length: usize,
    pub showing_options_menu: bool,
    pub program_options: ProgramOptions,
    pub update_status: AutoUpdateStatus,
    pub update_thread: Cell<Option<JoinHandle<Result<Status, String>>>>,
    pub open_modulus: i32,
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
        }
    }
}
