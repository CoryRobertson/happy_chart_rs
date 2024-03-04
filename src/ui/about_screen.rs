use crate::state::happy_chart_state::HappyChartState;
use crate::{BUILD_TIMESTAMP, GIT_DESCRIBE};
use egui::Ui;
use self_update::cargo_crate_version;

/// About page info such as build date, program license, and other fun stats that are slightly extraneous
#[tracing::instrument(skip(about_page_ui, app))]
pub fn draw_about_page(about_page_ui: &mut Ui, app: &mut HappyChartState) {
    about_page_ui.heading("Happy Chart");
    about_page_ui.label("A multi-purpose journaling software.");
    about_page_ui.separator();
    about_page_ui.label("Authors: Cory Robertson");
    about_page_ui.label("License: GPL-3.0");
    about_page_ui.horizontal(|about_page_ui| {
        about_page_ui.label("Github repository:");
        about_page_ui.hyperlink("https://github.com/CoryRobertson/happy_chart_rs");
    });
    about_page_ui.separator();
    about_page_ui.label(format!("Cargo crate version: {}", cargo_crate_version!()));
    about_page_ui.label(format!("Git describe: {}", GIT_DESCRIBE));
    about_page_ui.label(format!("BUILD_TIMESTAMP: {}", BUILD_TIMESTAMP));

    about_page_ui.separator();
    about_page_ui.label(format!("Last backup date: {}", app.last_backup_date));
    about_page_ui.label(format!("Last open date: {}", app.last_open_date));
    about_page_ui.label(format!(
        "Auto update seen version: {}",
        app.auto_update_seen_version.clone().unwrap_or_default()
    ));
    about_page_ui.label(format!(
        "Auto update status: {}",
        &app.update_status.to_text()
    ));

    about_page_ui.separator();

    if about_page_ui.button("Close").clicked() {
        app.showing_about_page = false;
    }
}
