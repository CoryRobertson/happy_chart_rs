use crate::prelude::HappyChartState;
use crate::NOTE_OLD_NUM_DAYS;
use chrono::Local;
use egui::{Color32, RichText, Ui};

#[tracing::instrument(skip_all)]
pub fn draw_note_edit_screen(ui: &mut Ui, app: &mut HappyChartState) {
    if let Some(index) = app.note_edit_selected {
        if let Some(note) = app.days.get_mut(index) {
            ui.label(note.to_string());
            ui.separator();
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
            ui.add_space(8.0);
            if ui.button("Set selected moods").on_hover_text("Sets moods that are currently selected from the mood selection screen to this day stat").clicked() {
                *note.get_moods_mut() = app.mood_selection_list.clone();
            }
            ui.add_space(8.0);
            if ui.button("Set selected activities").on_hover_text("Sets activities that are currently selected from the activities selection screen to this day stat").clicked() {
                *note.get_activities_mut() = app.ui_states.activity_ui_state.added_activity_list.get_activity_list().clone();
            }
        }
    }

    ui.separator();
    if ui.button("Close edit screen").clicked() {
        app.note_edit_selected = None;
    }
}
