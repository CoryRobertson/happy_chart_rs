
use eframe::epaint::Color32;

/// Color getter for text when displayed dynamically, not including font color for ui text.
pub fn get_text_color() -> Color32 {
    Color32::from_rgb(255,255,255)
}

/// Color getter for line color for graph between points
pub fn get_line_color() -> Color32 {
    Color32::from_rgb(100,100,100)
}

/// Color getter for info window that shows up when the user mouses over a point on the graph
pub fn get_info_window_color() -> Color32 {
    Color32::from_rgb(100,100,100)
}

/// Color for each point on graph, rating determines color, higher rating = closer to green, lower = closer to red
/// Rating is expected to cap out at 100.0
pub fn get_shape_color_from_rating(rating: f32) -> Color32 {

    let new_rating = rating / 100.0;

    let red: u8 = (100.0/new_rating) as u8;
    let green: u8 = (new_rating * 255.0) as u8;
    let blue: u8 = (new_rating * 50.0) as u8;

    Color32::from_rgb(red,green,blue)
}
