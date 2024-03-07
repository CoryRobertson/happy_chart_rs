use crate::options::color_setting::ColorSettings;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProgramOptions {
    pub graph_x_scale: f32,
    pub graph_y_scale: f32,
    pub x_offset: f32,
    pub draw_day_lines: bool,
    pub x_offset_slider_speed: f32,
    pub day_line_height_offset: f32,
    pub day_stat_height_offset: f32,
    /// The radius that is used to determine if the user is mousing over a day stat
    pub mouse_over_radius: f32,
    pub daystat_circle_outline_radius: f32,
    pub daystat_circle_size: f32,
    pub draw_daystat_circles: bool,
    pub draw_daystat_lines: bool,
    /// Mostly unused, but most likely will be used to determine if we should try to update the program every N number of launches
    pub update_modulus: i32,
    pub color_settings: ColorSettings,
    pub backup_save_path: PathBuf,
    /// Days to elapse FULLY between automatically backing up the program state to a zip
    pub auto_backup_days: i32,
    /// Number of days FULLY elapsed before a backup is considered stale and will be deleted when there are enough backups present
    pub backup_age_keep_days: i32,
    /// The minimum number of backups before we try to automatically clean up stale backups
    pub number_of_kept_backups: i32,
    /// Draw a different color outline for day stats within a streak of time
    pub show_streak: bool,
    /// The gap in hours a streak is considered valid
    pub streak_leniency: u32,
    /// Boolean for having all update list errors be ignored relating to getting the release list.
    /// This is so the user can not be annoyed by the error if they commonly use the program without an internet connection.
    pub disable_update_list_error_showing: bool,

    /// Move the day lines in the graph relative to the UI delta, making it move with how much stuff the user has written
    pub move_day_lines_with_ui: bool,

    pub encrypt_save_file: bool,

    pub do_opening_animation: bool,
}

impl Default for ProgramOptions {
    #[tracing::instrument]
    fn default() -> Self {
        Self {
            graph_x_scale: 1.0,
            graph_y_scale: 2.7,
            x_offset: 0.0,
            draw_day_lines: false,
            x_offset_slider_speed: 0.1,
            day_line_height_offset: 0.0,
            day_stat_height_offset: 0.0,
            mouse_over_radius: 20.0,
            daystat_circle_outline_radius: 5.2, // on my screen 5.2 looks pretty good, probably worth testing on other displays eventually
            daystat_circle_size: 4.0,
            draw_daystat_circles: true,
            draw_daystat_lines: true,
            update_modulus: -1,
            color_settings: ColorSettings::default(),
            backup_save_path: PathBuf::from("./backups/"),
            auto_backup_days: -1,
            backup_age_keep_days: -1,
            number_of_kept_backups: -1,
            show_streak: true,
            streak_leniency: 36,
            disable_update_list_error_showing: false,
            move_day_lines_with_ui: true,
            encrypt_save_file: false,
            do_opening_animation: true,
        }
    }
}
