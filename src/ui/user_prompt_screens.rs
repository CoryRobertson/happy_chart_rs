use crate::common::set_file_logging_state;
use crate::prelude::HappyChartState;
use crate::ui::options_menu::draw_backup_settings_options_menu;
use crate::LOG_FILE_NAME;
use egui::Context;

#[tracing::instrument(skip_all)]
pub fn draw_user_prompts(ctx: &Context, app: &mut HappyChartState) {
    // we should prompt the user to try features if they have specific thresholds of number of days logged in the program.
    // we also use an if else chain, so we only show the user one of these at a time, so it doesn't look spammy

    if !app.program_options.user_prompts.tried_logging
        && app.days.len() > 5
        && !app.program_options.log_to_file
    {
        egui::Window::new("Logging").show(ctx,|ui| {
            ui.label(format!("Happy chart supports logging info and errors to a file named {}", LOG_FILE_NAME));
            ui.label("The log does its best to not expose any identifying information that I would consider private, however you are free to disable it, or check yourself what data it logs.");
            ui.label("If you see any information that should not be logged due to any reason please make an issue on the github page here:");
            ui.hyperlink("https://github.com/CoryRobertson/happy_chart_rs");
            if ui.button("Enable logging").clicked() {
                app.program_options.log_to_file = true;
                set_file_logging_state(app.program_options.log_to_file);
                app.program_options.user_prompts.tried_logging = true;
            }

            if ui.button("Dismiss").clicked() {
                app.program_options.user_prompts.tried_logging = true;
            }
        });
    } else if !app.program_options.user_prompts.tried_backups && app.days.len() > 10 {
        egui::Window::new("Try backups").show(ctx,|ui| {
            ui.label("Happy chart supports taking save file backups, it is completely optional but very much recommended. \
            The storage amount used is very minimal as they are compressed. \
            It is also recommended to store these backup files on at least one other device.");
            draw_backup_settings_options_menu(ui,app,ctx);
            if ui.button("Dismiss").clicked() {
                app.program_options.user_prompts.tried_backups = true;
            }
        });
    }
}
