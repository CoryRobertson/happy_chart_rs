use crate::prelude::HappyChartState;
use crate::NOTE_OLD_NUM_DAYS;
use chrono::Local;
use egui::{Color32, RichText, Ui};

#[tracing::instrument(skip_all)]
pub fn draw_note_edit_screen(ui: &mut Ui, app: &mut HappyChartState) {
    if let Some(index) = app.note_edit_selected {
        if let Some(note) = app.days.get_mut(index) {
            ui.label(note.to_string());
            if note
                .get_date()
                .signed_duration_since(Local::now())
                .num_days()
                .abs()
                > NOTE_OLD_NUM_DAYS as i64
            {
                ui.label(RichText::new(format!("This note is older than {} days, it is not recommended to edit old notes as your memory of them may not be representative.", NOTE_OLD_NUM_DAYS)).color(Color32::LIGHT_RED));
            }
            ui.horizontal(|ui| {
                ui.label("Rating:");
                ui.add(egui::Slider::new(note.modify_rating(), 0.0..=100.0));
            });
        }
    }

    if ui.button("Close edit screen").clicked() {
        app.note_edit_selected = None;
    }
}
