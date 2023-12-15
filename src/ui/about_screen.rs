use crate::state::happy_chart_state::HappyChartState;
use crate::{BUILD_TIMESTAMP, GIT_DESCRIBE};
use egui::Ui;
use self_update::cargo_crate_version;

/// About page info such as build date, program license, and other fun stats that are slightly extraneous
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
    about_page_ui.label(format!("Day stats recorded: {}", app.days.len()));
    about_page_ui.label(format!(
        "Average sunday: {:.0}",
        app.stats.avg_weekdays.avg_sunday
    ));
    about_page_ui.label(format!(
        "Average monday: {:.0}",
        app.stats.avg_weekdays.avg_monday
    ));
    about_page_ui.label(format!(
        "Average tuesday: {:.0}",
        app.stats.avg_weekdays.avg_tuesday
    ));
    about_page_ui.label(format!(
        "Average wednesday: {:.0}",
        app.stats.avg_weekdays.avg_wednesday
    ));
    about_page_ui.label(format!(
        "Average thursday: {:.0}",
        app.stats.avg_weekdays.avg_thursday
    ));
    about_page_ui.label(format!(
        "Average friday: {:.0}",
        app.stats.avg_weekdays.avg_friday
    ));
    about_page_ui.label(format!(
        "Average saturday: {:.0}",
        app.stats.avg_weekdays.avg_saturday
    ));
    about_page_ui.label(format!(
        "Longest streak {}",
        app.stats.longest_streak.longest_streak
    ));
    about_page_ui.label(format!(
        "Streak start-end {}-{}",
        app.stats.longest_streak.streak_start_index, app.stats.longest_streak.streak_end_index
    ));
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
