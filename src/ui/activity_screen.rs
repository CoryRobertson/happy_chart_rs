use crate::prelude::HappyChartState;
use crate::state::activities::Activity;
use egui::{Context, Ui};
use tracing::info;

#[tracing::instrument(skip_all)]
pub fn draw_activity_selector_screen(ui: &mut Ui, _ctx: &Context, app: &mut HappyChartState) {
    if app.ui_states.activity_ui_state.edit_mode {
        if app.ui_states.activity_ui_state.add_or_remove_mode {
            ui.horizontal(|ui| {
                ui.label("Activity name:");
                ui.text_edit_singleline(&mut app.ui_states.activity_ui_state.activity_creat_text);
                if ui.button("Add new activity").clicked() {
                    app.program_options
                        .activity_list
                        .add_new_activity(Activity::new(
                            app.ui_states
                                .activity_ui_state
                                .activity_creat_text
                                .as_str()
                                .trim(),
                        ));
                }
            });
        }

        if !app.ui_states.activity_ui_state.add_or_remove_mode {
            for activity in app
                .program_options
                .activity_list
                .get_activity_list()
                .clone()
            {
                ui.horizontal(|ui| {
                    ui.label(format!("Remove {}", activity.get_activity_name()));
                    if ui.button("X").clicked() {
                        app.program_options.activity_list.remove_activity(&activity);
                    }
                });
            }
        }
    }

    if !app
        .program_options
        .activity_list
        .get_activity_list()
        .is_empty()
    {
        ui.separator();
        ui.label("Activity list");
        let row_width = 4;
        egui::Grid::new("Activity list grid")
            .striped(true)
            .show(ui, |ui| {
                for (index, activity) in app
                    .program_options
                    .activity_list
                    .get_activity_list()
                    .iter()
                    .enumerate()
                {
                    if app
                        .ui_states
                        .activity_ui_state
                        .added_activity_list
                        .get_activity_list()
                        .contains(activity)
                    {
                        ui.label(activity.get_activity_name());
                    } else if ui.button(activity.get_activity_name()).clicked() {
                        app.ui_states
                            .activity_ui_state
                            .added_activity_list
                            .add_new_activity(activity.clone());
                    }
                    if index != 0 && index % row_width == (row_width - 1) {
                        ui.end_row();
                    }
                }
            });
    }

    if !app
        .ui_states
        .activity_ui_state
        .added_activity_list
        .get_activity_list()
        .is_empty()
    {
        ui.separator();
        ui.label("Selected activities:");
        for activity in app
            .ui_states
            .activity_ui_state
            .added_activity_list
            .get_activity_list()
            .clone()
        {
            if ui.button(activity.get_activity_name()).clicked() {
                app.ui_states
                    .activity_ui_state
                    .added_activity_list
                    .remove_activity(&activity);
            }
        }
    }

    ui.separator();

    ui.horizontal(|ui| {
        if app.ui_states.activity_ui_state.show_activity_screen && ui.button("Close").clicked() {
            info!("Activity selection screen closed");
            app.ui_states.activity_ui_state.show_activity_screen = false;
        }
        ui.checkbox(&mut app.ui_states.activity_ui_state.edit_mode, "Edit Mode");
        if app.ui_states.activity_ui_state.edit_mode {
            let add_or_remove_mode_text = if app.ui_states.activity_ui_state.add_or_remove_mode {
                "Add mode"
            } else {
                "Remove mode"
            };
            if ui.button(add_or_remove_mode_text).clicked() {
                app.ui_states.activity_ui_state.add_or_remove_mode =
                    !app.ui_states.activity_ui_state.add_or_remove_mode;
            }
        }
    });
}
