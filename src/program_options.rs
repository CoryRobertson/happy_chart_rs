use crate::color_setting::ColorSettings;
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
}

impl Default for ProgramOptions {
    fn default() -> Self {
        Self {
            graph_x_scale: 1.0,
            graph_y_scale: 1.0,
            x_offset: 0.0,
            draw_day_lines: false,
            x_offset_slider_speed: 0.1,
            day_line_height_offset: 0.0,
            day_stat_height_offset: 0.0,
            mouse_over_radius: 20.0,
            daystat_circle_outline_radius: 5.0,
            daystat_circle_size: 4.0,
            draw_daystat_circles: true,
            draw_daystat_lines: true,
            update_modulus: -1,
            color_settings: ColorSettings::default(),
            backup_save_path: PathBuf::from("./backups/"),
            auto_backup_days: -1,
            backup_age_keep_days: -1,
            number_of_kept_backups: -1,
        }
    }
}
