use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProgramOptions {
    pub x_offset_slider_speed: f32,
    pub day_line_height_offset: f32,
    pub day_stat_height_offset: f32,
    pub mouse_over_radius: f32,
}

impl Default for ProgramOptions {
    fn default() -> Self {
        Self {
            x_offset_slider_speed: 0.1,
            day_line_height_offset: 0.0,
            day_stat_height_offset: 0.0,
            mouse_over_radius: 20.0,
        }
    }
}
