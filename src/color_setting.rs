use eframe::epaint::Color32;
use serde::{Deserialize, Serialize};

/// Color getter for text when displayed dynamically, not including font color for ui text.
#[deprecated]
#[allow(dead_code)]
pub const fn get_text_color() -> Color32 {
    Color32::from_rgb(255, 255, 255)
}

/// Color getter for line color for graph between points
#[deprecated]
#[allow(dead_code)]
pub const fn get_line_color() -> Color32 {
    Color32::from_rgb(100, 100, 100)
}

/// Color getter for info window that shows up when the user mouses over a point on the graph
#[deprecated]
#[allow(dead_code)]
pub const fn get_info_window_color() -> Color32 {
    Color32::from_rgb(100, 100, 100)
}

/// Color for each point on graph, rating determines color, higher rating = closer to green, lower = closer to red
/// Rating is expected to cap out at 100.0
pub fn get_shape_color_from_rating(rating: f32) -> Color32 {
    let new_rating = rating / 100.0;

    let red: u8 = (100.0 / new_rating) as u8;
    let green: u8 = (new_rating * 255.0) as u8;
    let blue: u8 = (new_rating * 50.0) as u8;

    Color32::from_rgb(red, green, blue)
}

/// Color for the lines that show each day, when they are turned on.
#[deprecated]
#[allow(dead_code)]
pub fn get_day_line_color() -> Color32 {
    Color32::from_rgba_unmultiplied(50, 50, 50, 100)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ColorSettings {
    /// Color getter for text when displayed dynamically, not including font color for ui text.
    pub text_color: Color32,
    /// Color getter for line color for graph between points
    pub line_color: Color32,
    /// Color getter for info window that shows up when the user mouses over a point on the graph
    pub info_window_color: Color32,
    /// Color for the lines that show each day, when they are turned on.
    pub day_line_color: Color32,
}

impl Default for ColorSettings {
    fn default() -> Self {
        Self {
            text_color: Color32::from_rgb(255, 255, 255),
            line_color: Color32::from_rgb(100, 100, 100),
            info_window_color: Color32::from_rgb(100, 100, 100),
            day_line_color: Color32::from_rgba_unmultiplied(50, 50, 50, 100),
        }
    }
}