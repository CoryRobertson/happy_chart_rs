use crate::auto_update_status::AutoUpdateStatus;
use crate::common::{backup_program_state, toggle_ui_compact};
use crate::options::color_setting::ColorSettings;
use crate::options::program_options::ProgramOptions;
use crate::state::happy_chart_state::HappyChartState;
use chrono::Local;
use egui::{Context, Ui};
use self_update::Status;

/// Draw an indicator in the options menu for if an update is taking place, or needed
pub(crate) fn options_update_thread_block(options_panel_ui: &mut Ui, app: &mut HappyChartState) {
    // update thread block, handles showing spinner, and checking if the update is done
    let update_thread = app.update_thread.replace(None);
    match update_thread {
        None => {}
        Some(thread) => {
            if thread.is_finished() {
                if let Ok(res) = thread.join() {
                    match res {
                        Ok(status) => match status {
                            Status::UpToDate(ver) => {
                                app.update_status = AutoUpdateStatus::UpToDate(ver);
                            }
                            Status::Updated(ver) => {
                                app.update_status = AutoUpdateStatus::Updated(ver);
                            }
                        },
                        Err(err) => {
                            app.update_status = AutoUpdateStatus::Error(err);
                        }
                    }
                }
            } else {
                app.update_thread.replace(Some(thread));
                app.update_status = AutoUpdateStatus::Checking;
                options_panel_ui.spinner();
            }
        }
    }
}

/// Color options collapsing menu
pub(crate) fn draw_color_options_menu(options_panel_ui: &mut Ui, app: &mut HappyChartState) {
    options_panel_ui.collapsing("Color options", |ui| {
        ui.horizontal(|ui| {
            ui.color_edit_button_srgba(&mut app.program_options.color_settings.line_color)
                .on_hover_text("Line color");
            ui.color_edit_button_srgba(&mut app.program_options.color_settings.day_line_color)
                .on_hover_text("Day line color");
            // TODO: text color doesnt work cause we use the foreground color for this, probably not a good idea to let the user change this normally yet until I think of a way to do it in a pretty way
            // ui.color_edit_button_srgba(&mut self.program_options.color_settings.text_color).on_hover_text("Text Color");
            ui.color_edit_button_srgba(&mut app.program_options.color_settings.info_window_color)
                .on_hover_text("Info window color");
        });

        if ui.button("Reset colors to defaults").clicked() {
            app.program_options.color_settings = ColorSettings::default();
        }
    });
}

/// Graphing options collapsing menu
pub(crate) fn draw_graphing_options_menu(options_panel_ui: &mut Ui, app: &mut HappyChartState) {
    options_panel_ui.collapsing("Graphing options", |options_panel_ui| {
        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Display day lines: ");

            toggle_ui_compact(options_panel_ui, &mut app.program_options.draw_day_lines);
        });

        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Graph X Scale: ");
            options_panel_ui
                .add(egui::Slider::new(
                    &mut app.program_options.graph_x_scale,
                    0.01..=10.0,
                ))
                .on_hover_text("Multiplier used to scale the graph on the X axis.");
        });

        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Graph Y Scale: ");
            options_panel_ui
                .add(egui::Slider::new(
                    &mut app.program_options.graph_y_scale,
                    0.5..=5.0,
                ))
                .on_hover_text("Multiplier used to scale the graph on the Y axis.");
        });

        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("X Offset: ");
            options_panel_ui
                .add(
                    egui::DragValue::new(&mut app.program_options.x_offset)
                        .speed(app.program_options.x_offset_slider_speed),
                )
                .on_hover_text("Amount of units to shift the graph on the X axis.");
        });

        // x offset slider speed
        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("X offset slider speed:");
            options_panel_ui.add(
                egui::DragValue::new(&mut app.program_options.x_offset_slider_speed).speed(0.1),
            );
        });

        // day line height
        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Day line height:");
            options_panel_ui.add(
                egui::DragValue::new(&mut app.program_options.day_line_height_offset).speed(0.1),
            );
        });
    });
}

/// Day stat options collapsing menu
pub(crate) fn draw_stat_drawing_options_menu(options_panel_ui: &mut Ui, app: &mut HappyChartState) {
    options_panel_ui.collapsing("Stat drawing options", |options_panel_ui| {
        // mouse over radius
        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Stat mouse over radius:");
            options_panel_ui
                .add(egui::DragValue::new(&mut app.program_options.mouse_over_radius).speed(0.1));
        });

        // stat height offset
        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Stat height offset:");
            options_panel_ui.add(
                egui::DragValue::new(&mut app.program_options.day_stat_height_offset).speed(0.1),
            );
        });

        // day stat circle sizes
        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Stat circle radius:");
            options_panel_ui
                .add(egui::DragValue::new(&mut app.program_options.daystat_circle_size).speed(0.1));
        });
        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Stat circle outline radius:");
            options_panel_ui.add(
                egui::DragValue::new(&mut app.program_options.daystat_circle_outline_radius)
                    .speed(0.1),
            );
        });
        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.checkbox(
                &mut app.program_options.draw_daystat_circles,
                "Draw stat circles",
            );
            options_panel_ui.checkbox(
                &mut app.program_options.draw_daystat_lines,
                "Draw stat lines",
            );
        });
    });
}

/// Backup settings collapsing menu
pub(crate) fn draw_backup_settings_options_menu(
    options_panel_ui: &mut Ui,
    app: &mut HappyChartState,
    ctx: &Context,
) {
    options_panel_ui.collapsing("Backup options", |options_panel_ui| {
        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Backup folder ");
            if options_panel_ui.button("Browse path").on_hover_text(format!("Current backup folder: {:?}", app.program_options.backup_save_path.clone().into_os_string())).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_directory("./")
                    .set_title("Set the location where a backup will be stored")
                    .pick_folder() {
                    app.program_options.backup_save_path = path;
                }
            }

        });

        if options_panel_ui.button("Reset backup path").clicked() {
            app.program_options.backup_save_path = ProgramOptions::default().backup_save_path;
        }

        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Auto backup day count: ");
            options_panel_ui.add(
                egui::DragValue::new(&mut app.program_options.auto_backup_days)
            ).on_hover_text("The number of days to elapse between auto backups, if less than 0, no automatic backups will take place.");
        });

        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Backup age before removal: ");
            options_panel_ui.add(
                egui::DragValue::new(&mut app.program_options.backup_age_keep_days)
            ).on_hover_text("The number of days to elapse before deleting a backup, < 0 = never remove");
        });

        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Number of stale backups before removal: ");
            options_panel_ui.add(
                egui::DragValue::new(&mut app.program_options.number_of_kept_backups)
            ).on_hover_text("The minimum number of stale backups needed to be present in the backups folder before the program will remove any, -1 for disabled.");
        });

        if options_panel_ui.button("Backup program state").on_hover_text("Compress the save state and the last session data into a zip file titled with the current date.").clicked() {
            backup_program_state(ctx, app, true);
            app.last_backup_date = Local::now();
        }
    });
}