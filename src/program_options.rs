use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProgramOptions {
    pub graph_x_scale: f32,
    pub graph_y_scale: f32,
    pub x_offset: i32,
    pub drawing_lines: bool,
    pub x_offset_slider_speed: f32,
    pub day_line_height_offset: f32,
    pub day_stat_height_offset: f32,
    pub mouse_over_radius: f32,
}

impl Default for ProgramOptions {
    fn default() -> Self {
        Self {
            graph_x_scale: 1.0,
            graph_y_scale: 1.0,
            x_offset: 0,
            drawing_lines: false,
            x_offset_slider_speed: 0.1,
            day_line_height_offset: 0.0,
            day_stat_height_offset: 0.0,
            mouse_over_radius: 20.0,
        }
    }
}
