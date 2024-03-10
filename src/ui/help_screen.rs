use crate::state::happy_chart_state::HappyChartState;
use egui::Ui;

/// Draw a collapsing help menu that has a lot of useful information to make usage of the program more smooth
#[tracing::instrument(skip_all)]
pub fn draw_help_dropdown(ui: &mut Ui, _app: &HappyChartState) {
    ui.collapsing("Help, Tips, and Controls", |help_ui| {
        help_ui.label("Right click and drag to move the journal entry graph laterally");
        help_ui.add_space(8.0);
        help_ui.label("Right click + left click to scale and move the journal graph");
        help_ui.add_space(8.0);
        help_ui.label("Control + left click a journal entry to change things about the entry");
        help_ui.add_space(8.0);
        help_ui.label("Enable save file backups in the settings menu so you can be sure you wont lose your data");
    });
}
