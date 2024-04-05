use crate::common::auto_update_status::AutoUpdateStatus;
use crate::common::backup::backup_program_state;
use crate::common::save::{read_last_session_save_file, read_save_file, save_program_state};
use crate::common::update::get_release_list;
use crate::state::error_states::HappyChartError;
use crate::state::happy_chart_state::HappyChartState;
use chrono::Local;
use eframe::egui;
use eframe::epaint::ColorImage;
use egui::{Context, ViewportCommand};
use self_update::cargo_crate_version;
use std::sync::Arc;
use tracing::{error, info};

pub mod auto_update_status;
pub mod backup;
pub mod color;
pub mod encryption;
pub mod export;
pub mod last_session;
pub mod math;
pub mod mood_tag;
pub mod save;
pub mod update;

/// Quit function run when the user clicks the quit button
#[tracing::instrument(skip(ctx, app))]
pub fn quit(ctx: &Context, app: &HappyChartState) -> Result<(), HappyChartError> {
    info!("Quit program sequence started");
    save_program_state(ctx, app)?;

    ctx.send_viewport_cmd(ViewportCommand::Close);
    info!("Quit program sequence completed");
    Ok(())
}

/// First load governs error states on its own, no need to read output
#[tracing::instrument(skip(app, ctx))]
pub fn first_load(app: &mut HappyChartState, ctx: &Context, load_save: bool) {
    info!("Running program startup");
    // all data we need to read one time on launch, all of this most of the time is unchanging throughout usage of the program, so it can only be recalculated on launch
    // for example, day quality averages do not need to change between launches
    app.first_load = false;

    if load_save {
        match read_save_file() {
            Ok(save_file_days) => {
                app.days = save_file_days;
            }
            Err(err) => {
                error!("Error reading save file: {}", err);
                app.error_states.push(err);
            }
        }
    }

    app.days.sort_by(|day1, day2| {
        day1.get_date()
            .timestamp()
            .cmp(&day2.get_date().timestamp())
    });

    app.starting_length = app.days.len();

    if load_save {
        let ls = read_last_session_save_file();
        app.open_modulus = ls.open_modulus;
        app.program_options = ls.program_options;
        app.last_open_date = ls.last_open_date;
        app.last_backup_date = ls.last_backup_date;
        app.open_animation_animating = app.program_options.do_opening_animation;
        if let Some(ver) = ls.last_version_checked {
            app.auto_update_seen_version = Some(ver);
        }
        app.tutorial_state = ls.tutorial_state;
        if Local::now()
            .signed_duration_since(ls.last_open_date)
            .num_hours()
            >= 12
        {
            match get_release_list() {
                Ok(list) => {
                    if let Some(release) = list.first() {
                        if let Ok(greater_bump) = self_update::version::bump_is_greater(
                            cargo_crate_version!(),
                            &release.version,
                        ) {
                            if greater_bump {
                                info!(
                                    "Update available! {} {} {}",
                                    release.name, release.version, release.date
                                );
                                app.update_available = Some(release.clone());
                                app.update_status = AutoUpdateStatus::OutOfDate;
                            } else {
                                info!("No update available.");
                                app.update_status =
                                    AutoUpdateStatus::UpToDate(cargo_crate_version!().to_string());
                            }
                        }
                    }
                }
                Err(err) => {
                    error!("Error getting release list: {}", err);
                    if !app.program_options.disable_update_list_error_showing {
                        app.error_states
                            .push(HappyChartError::UpdateReleaseList(err));
                    }
                }
            }
        }

        // check if user last backup day is +- 3 hours between the margin of their auto backup day count
        if app.program_options.auto_backup_days > -1
            && Local::now()
                .signed_duration_since(ls.last_backup_date)
                .num_days()
                > i64::from(app.program_options.auto_backup_days)
        {
            match backup_program_state(ctx, app, false) {
                Ok(_) => {
                    app.last_backup_date = Local::now();
                }
                Err(err) => {
                    error!("Error backing up program state: {:?}", err);
                    app.error_states.push(err);
                }
            }
        }
    }

    app.remove_old_backup_files();
    app.stats
        .calc_all_stats(&app.days, app.program_options.streak_leniency);
}

#[tracing::instrument(skip(image))]
pub fn handle_screenshot_event(image: &Arc<ColorImage>) {
    info!("Saving screenshot");

    match rfd::FileDialog::new()
        .add_filter("Image", &["png", "jpeg", "jpg", "bmp", "tiff"])
        .save_file()
    {
        None => {
            info!("No save path selected");
        }
        Some(path) => {
            match image::save_buffer(
                path,
                image.as_raw(),
                u32::try_from(image.width()).unwrap_or(u32::MAX),
                u32::try_from(image.height()).unwrap_or(u32::MAX),
                image::ColorType::Rgba8,
            ) {
                Ok(_) => {
                    info!("Screenshot save successful");
                }
                Err(err) => {
                    error!("Screenshot save error: {}", err);
                }
            }
        }
    }
}

// thank you online example <3
#[tracing::instrument(skip(ui))]
pub fn toggle_ui_compact(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }
    response
}
