use eframe::epaint::Color32;
use egui::Ui;
use std::time::SystemTime;

#[tracing::instrument]
pub fn get_tutorial_highlight_glowing_color(offset: u8) -> Color32 {
    let now = SystemTime::now();

    let diff = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |diff| diff.as_secs());

    match (diff + u64::from(offset)) % 3 {
        0 => Color32::GRAY,
        1 => Color32::LIGHT_GRAY,
        _ => Color32::WHITE,
    }
}

#[tracing::instrument(skip_all)]
pub fn tutorial_button_colors(ui: &mut Ui) {
    ui.style_mut().visuals.widgets.inactive.bg_fill = get_tutorial_highlight_glowing_color(0);
    ui.style_mut().visuals.widgets.inactive.fg_stroke.color =
        get_tutorial_highlight_glowing_color(2);
    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = get_tutorial_lowlight_glowing_color(0);
}

#[tracing::instrument]
pub fn get_tutorial_lowlight_glowing_color(offset: u8) -> Color32 {
    let now = SystemTime::now();

    let diff = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |diff| diff.as_secs());

    match (diff + u64::from(offset)) % 3 {
        0 => Color32::from_gray(90),
        1 => Color32::from_gray(60),
        2 => Color32::from_gray(40),
        _ => Color32::DARK_GRAY,
    }
}
