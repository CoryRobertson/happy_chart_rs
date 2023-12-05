#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod color_setting;
mod daystat;
mod improved_daystat;
mod last_session;
mod program_options;

mod auto_update_status;

mod common;

mod state_stats;

mod happy_chart_state;

const GIT_DESCRIBE: &str = env!("VERGEN_GIT_DESCRIBE");
const BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

use std::sync::Arc;
use crate::auto_update_status::AutoUpdateStatus;
use crate::color_setting::ColorSettings;
use crate::common::{
    backup_program_state, distance, get_release_list, improved_calculate_x, quit,
    read_last_session_save_file, read_save_file, toggle_ui_compact, update_program,
};
use crate::egui::Layout;
use crate::happy_chart_state::HappyChartState;
use crate::improved_daystat::ImprovedDayStat;
use crate::program_options::ProgramOptions;
use chrono::{Days, Local};
use eframe::emath::Pos2;
use eframe::{egui, Frame, NativeOptions};
use egui::{Align2, Color32, ColorImage, Context, FontId, Rect, Rounding, Stroke, Ui, Vec2, ViewportBuilder, ViewportCommand};
use self_update::{cargo_crate_version, Status};

const SAVE_FILE_NAME: &str = "save.ser";
const NEW_SAVE_FILE_NAME: &str = "happy_chart_save.ser";
const LAST_SESSION_FILE_NAME: &str = "happy_chart_last_session.ser";

const BACKUP_FILENAME_PREFIX: &str = "happy_chart_backup_";

const MANUAL_BACKUP_SUFFIX: &str = "_manual";

const BACKUP_FILE_EXTENSION: &str = "zip";

fn main() {

    let window_size: Vec2 = read_last_session_save_file().window_size.into();

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size(window_size),
        ..Default::default()
    };

    eframe::run_native(
        "Happy Chart",
        native_options,
        Box::new(|cc| Box::new(HappyChartState::new(cc))),
    )
    .expect("Failed to run egui app");
}

fn first_load(app: &mut HappyChartState, ctx: &Context) {
    // all data we need to read one time on launch, all of this most of the time is unchanging throughout usage of the program, so it can only be recalculated on launch
    // for example, day quality averages do not need to change between launches
    app.first_load = false;
    app.days = read_save_file();

    app.days
        .sort_by(|day1, day2| day1.date.timestamp().cmp(&day2.date.timestamp()));

    app.starting_length = app.days.len();
    let ls = read_last_session_save_file();
    app.open_modulus = ls.open_modulus;
    app.program_options = ls.program_options;
    app.last_open_date = ls.last_open_date;
    app.last_backup_date = ls.last_backup_date;
    if let Some(ver) = ls.last_version_checked {
        app.auto_update_seen_version = Some(ver);
    }

    if Local::now()
        .signed_duration_since(ls.last_open_date)
        .num_hours()
        >= 12
    {
        if let Ok(list) = get_release_list() {
            if let Some(release) = list.first() {
                if let Ok(greater_bump) = self_update::version::bump_is_greater(
                    cargo_crate_version!(),
                    &release.version,
                ) {
                    if greater_bump {
                        println!(
                            "Update available! {} {} {}",
                            release.name, release.version, release.date
                        );
                        app.update_available = Some(release.clone());
                        app.update_status = AutoUpdateStatus::OutOfDate;
                    } else {
                        println!("No update available.");
                        app.update_status =
                            AutoUpdateStatus::UpToDate(cargo_crate_version!().to_string());
                    }
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
        backup_program_state(ctx, app, false);
        app.last_backup_date = Local::now();
    }

    app.remove_old_backup_files();

    app.stats.avg_weekdays.calc_averages(&app.days);
}

fn handle_screenshot_event(image: &Arc<ColorImage>) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Image", &["png", "jpeg", "jpg", "bmp", "tiff"])
        .save_file()
    {
        image::save_buffer(
            path,
            image.as_raw(),
            image.width() as u32,
            image.height() as u32,
            image::ColorType::Rgba8,
        )
            .unwrap();
    }
}

fn main_screen_button_ui(central_panel_ui: &mut Ui, app: &mut HappyChartState) {
    central_panel_ui.horizontal(|ui| {
        ui.label("Rating: ");
        ui.add(egui::Slider::new(&mut app.rating, 0.0..=100.0))
            .on_hover_text("The rating of the given day to be saved to the graph point.");
    });

    central_panel_ui.horizontal(|ui| {
        ui.label("Note: ");
        ui.text_edit_multiline(&mut app.note_input)
            .on_hover_text("The note to add to the next added graph point.");
    });

    if central_panel_ui.button("Add day").clicked() {
        app.days.push(ImprovedDayStat {
            rating: app.rating as f32,
            date: ImprovedDayStat::get_current_time_system(),
            note: app.note_input.clone(),
        });
        app.stats.avg_weekdays.calc_averages(&app.days);
        println!(
            "day added with rating {} and date {}",
            app.rating,
            ImprovedDayStat::get_current_time_system()
        );
    }

    if central_panel_ui.button("Remove day").clicked() && !app.days.is_empty() {
        app.days.remove(app.days.len() - 1);
        app.stats.avg_weekdays.calc_averages(&app.days);
    }

    central_panel_ui.horizontal(|ui| {
        ui.label("Search: ");
        ui.add_sized(Vec2::new(120.0,20.0),egui::widgets::text_edit::TextEdit::singleline(&mut app.filter_term));
    });
}

fn click_drag_zoom_detection(central_panel_ui: &mut Ui, app: &mut HappyChartState, pointer_interact_pos: Option<&Pos2>) {
    let within_day_lines = {
        let min_y: f32 = 220.0 - app.program_options.day_line_height_offset;
        pointer_interact_pos.map_or(false, |pos| pos.y >= min_y)
    };

    if within_day_lines {
        let right_click_down = central_panel_ui.input(|i| i.pointer.secondary_down());

        let left_click_down = central_panel_ui.input(|i| i.pointer.primary_down());

        // if right click is down, allow the xoffset to be moved
        if right_click_down {
            let drag_delta = central_panel_ui.input(|i| i.pointer.delta());

            app.program_options.x_offset += drag_delta.x;

            // if both right click and left click are down, then we allow the x scale to be changed so the user can quickly zoom into areas on the graph
            if left_click_down {
                app.program_options.graph_x_scale += -drag_delta.y / 1000.0;
                app.program_options.x_offset += drag_delta.y * (10.0);
            }

            if app.program_options.graph_x_scale.is_sign_negative() {
                app.program_options.graph_x_scale = 0.001;
            }
        }
    }
}


/// Draw the lines that represent time itself, typically 24 hours
fn draw_day_lines(central_panel_ui: &mut Ui, app: &mut HappyChartState) {
    if app.program_options.draw_day_lines && app.days.len() > 1 {
        // range for calculating how many lines in both directions on the x axis
        let range = {
            if app.program_options.x_offset > 5000.0 {
                app.program_options.x_offset as i32
            } else {
                5000
            }
        };

        for i2 in -range..range {
            // make a fake day with the first day on the list as the first day, and add 24 hours to it each time in utc time to calculate where each line goes
            let line_points: [Pos2; 2] = {
                let d = app.days.get(0).unwrap();

                let fake_day = ImprovedDayStat {
                    rating: 0.0,
                    date: d.date.checked_add_days(Days::new(1)).unwrap_or_default(), // fake day that starts from where the first day is, with one day added
                    note: String::new(),
                };
                let y: f32 = 220.0 - app.program_options.day_line_height_offset;
                let x = {
                    let first_day = d;
                    let hours: f32 =
                        fake_day.get_hour_difference(first_day) as f32 / 3600.0; // number of hours compared to the previous point

                    let x: f32 = (hours * app.program_options.graph_x_scale) * i2 as f32;
                    x + app.program_options.x_offset
                };
                [Pos2::new(x, y), Pos2::new(x, 800.0)]
            };
            central_panel_ui.painter().line_segment(
                line_points,
                Stroke::new(2.0, app.program_options.color_settings.day_line_color),
            );
        }
    }
}

fn draw_stat_line_segments(central_panel_ui: &mut Ui, app: &mut HappyChartState) {
    let mut i = 0;
    let mut prev_x = 0.0;
    let mut prev_y = 0.0;
    // draw lines loop, bottom layer
    if app.program_options.draw_daystat_lines {
        for day in &app.days {
            let x: f32 = improved_calculate_x(
                &app.days,
                day,
                app.program_options.graph_x_scale,
                app.program_options.x_offset,
            );

            let y: f32 = day
                .rating
                .mul_add(-app.program_options.graph_y_scale, 500.0)
                - app.program_options.day_stat_height_offset;
            let points = [Pos2::new(prev_x, prev_y), Pos2::new(x, y)];

            if (prev_x != 0.0 && prev_y != 0.0) || i == 1 {
                // draw line segments connecting the dots
                central_panel_ui.painter().line_segment(
                    points,
                    Stroke::new(2.0, app.program_options.color_settings.line_color),
                );
            }

            i += 1;
            prev_x = x;
            prev_y = y;
        }
    }
}

fn draw_stat_circles(central_panel_ui: &mut Ui, app: &mut HappyChartState) {
    if app.program_options.draw_daystat_circles {
        for day in &app.days.clone() {
            let x: f32 = improved_calculate_x(
                &app.days,
                day,
                app.program_options.graph_x_scale,
                app.program_options.x_offset,
            );
            let y: f32 = day
                .rating
                .mul_add(-app.program_options.graph_y_scale, 500.0)
                - app.program_options.day_stat_height_offset;

            //draw circles on each coordinate point
            central_panel_ui.painter().circle_filled(
                Pos2::new(x, y),
                app.program_options.daystat_circle_outline_radius,
                Color32::BLACK,
            );

            let color = if !app.filter_term.is_empty() &&day.note.contains(&app.filter_term) {
                Color32::LIGHT_BLUE
            } else {
                color_setting::get_shape_color_from_rating(day.rating)
            };

            central_panel_ui.painter().circle_filled(
                Pos2::new(x, y),
                app.program_options.daystat_circle_size,
                color,
            );
        }
    }
}

fn draw_stat_mouse_over_info(central_panel_ui: &mut Ui, app: &mut HappyChartState, ctx: &egui::Context) {
    let mouse_pos = ctx.pointer_hover_pos().map_or_else(|| Pos2::new(0.0, 0.0), |a| a);
    let mut moused_over = false; // boolean used to know if we are already showing mouse over text, if so, not to render it if this is true
    // draw text loop, top most layer (mostly)
    for day in &app.days {
        let x: f32 = improved_calculate_x(
            &app.days,
            day,
            app.program_options.graph_x_scale,
            app.program_options.x_offset,
        );
        let y: f32 = day
            .rating
            .mul_add(-app.program_options.graph_y_scale, 500.0)
            - app.program_options.day_stat_height_offset;
        let rect_pos1 = Pos2::new(520.0, 10.0);
        let rect_pos2 = Pos2::new(770.0, 180.0);
        let text = day.to_string();

        let dist_max = app.program_options.mouse_over_radius; // maximum distance to consider a point being moused over

        if distance(mouse_pos.x, mouse_pos.y, x, y) < dist_max && !moused_over {
            // draw text near by each coordinate point
            moused_over = true;

            central_panel_ui.painter().text(
                Pos2::new(x + 20.0, y),
                Align2::LEFT_CENTER,
                &text,
                FontId::default(),
                app.program_options.color_settings.text_color, // color_setting::get_text_color(),
            );

            central_panel_ui.painter().rect_filled(
                Rect::from_two_pos(rect_pos1, rect_pos2),
                Rounding::from(20.0),
                app.program_options.color_settings.info_window_color,
            );
            central_panel_ui.style_mut().visuals.override_text_color =
                Option::from(app.program_options.color_settings.text_color);

            // info text to display in top right window
            let info_text: String = day.to_string();

            central_panel_ui.put(
                Rect::from_two_pos(rect_pos1, rect_pos2),
                egui::widgets::Label::new(&info_text),
            );
        }
    }
}

fn draw_auto_update_ui(central_panel_ui: &mut Ui, app: &mut HappyChartState, ctx: &Context) {
    if let Some(release) = &app.update_available {
        let should_show_update = match &app.auto_update_seen_version {
            None => {
                true
            }
            Some(ver) => {
                self_update::version::bump_is_greater(ver,&release.version).unwrap_or(true)
            }
        };
        if should_show_update {
            if central_panel_ui.button("Dismiss update").clicked() {
                app.auto_update_seen_version = Some(release.version.to_string());
            }
            if central_panel_ui.button("Update happy chart").clicked() {
                app.update_thread.replace(Some(update_program()));
                app.auto_update_seen_version = Some(release.version.to_string());
            }
            let mid_point_x = (ctx.screen_rect().width() / 2.0) - (250.0/2.0);
            let quarter_point_y = ctx.screen_rect().height() / 4.0;

            central_panel_ui.painter().rect_filled(
                Rect::from_two_pos(Pos2::new(mid_point_x, quarter_point_y), Pos2::new(mid_point_x + 250.0, quarter_point_y + 120.0)),
                Rounding::from(4.0),
                app.program_options.color_settings.info_window_color,
            );
            central_panel_ui.style_mut().visuals.override_text_color =
                Option::from(app.program_options.color_settings.text_color);

            central_panel_ui.put(
                Rect::from_two_pos(Pos2::new(mid_point_x, quarter_point_y), Pos2::new(mid_point_x + 250.0, quarter_point_y + 120.0)),
                egui::widgets::Label::new(format!("Update available:\n{}\nCurrent version:\nv{}\n\"Update happy chart\" to automagically update\nThis message will not display on next launch", release.name,cargo_crate_version!())),
            );

        }
    }
    central_panel_ui.horizontal(|ui| {
        if let Some(thread) = app.update_thread.get_mut() {
            if !thread.is_finished() {
                ui.label("Updating... ");
                ui.spinner();
            }
        }
    });
}

fn draw_quit_button(central_panel_ui: &mut Ui, app: &mut HappyChartState, ctx: &Context) {
    // quit button layout
    central_panel_ui.with_layout(Layout::bottom_up(egui::Align::BOTTOM), |ui| {
        if app.starting_length != app.days.len() {
            ui.visuals_mut().override_text_color = Option::from(Color32::RED);
        } else {
            ui.style_mut().visuals.override_text_color =
                Option::from(app.program_options.color_settings.text_color);
        }

        ui.horizontal(|ui| {
            let quit_button = ui.button("Save & Quit");

            if quit_button.clicked() {
                quit(ctx, app);
            }

            ui.style_mut().visuals.override_text_color =
                Option::from(app.program_options.color_settings.text_color);

            if !app.showing_options_menu && ui.button("Options").clicked() {
                app.showing_options_menu = true;
            }

            if !app.showing_about_page && ui.button("About").clicked() {
                app.showing_about_page = true;
            }

            if ui.button("Save Screenshot").clicked() {
                // frame.request_screenshot();
                ctx.send_viewport_cmd(ViewportCommand::Screenshot);
            }

            if quit_button.hovered() {
                ui.label(
                    egui::RichText::new(BUILD_TIMESTAMP)
                        .color(Color32::from_rgb(80, 80, 80)),
                );
                ui.label(
                    egui::RichText::new(GIT_DESCRIBE).color(Color32::from_rgb(80, 80, 80)),
                );
            }
        });
    });
}

fn options_update_thread_block(options_panel_ui: &mut Ui, app: &mut HappyChartState) {
    // update thread block, handles showing spinner, and checking if the update is done
    let update_thread = app.update_thread.replace(None);
    match update_thread {
        None => {}
        Some(thread) => {
            if thread.is_finished() {
                if let Ok(res) = thread.join() { match res {
                    Ok(status) => match status {
                        Status::UpToDate(ver) => {
                            app.update_status =
                                AutoUpdateStatus::UpToDate(ver);
                        }
                        Status::Updated(ver) => {
                            app.update_status = AutoUpdateStatus::Updated(ver);
                        }
                    },
                    Err(err) => {
                        app.update_status = AutoUpdateStatus::Error(err);
                    }
                } }
            } else {
                app.update_thread.replace(Some(thread));
                app.update_status = AutoUpdateStatus::Checking;
                options_panel_ui.spinner();
            }
        }
    }
}

fn draw_color_options_menu(options_panel_ui: &mut Ui, app: &mut HappyChartState) {
    options_panel_ui.collapsing("Color options", |ui| {
        ui.horizontal(|ui| {
            ui.color_edit_button_srgba(&mut app.program_options.color_settings.line_color)
                .on_hover_text("Line color");
            ui.color_edit_button_srgba(
                &mut app.program_options.color_settings.day_line_color,
            )
                .on_hover_text("Day line color");
            // TODO: text color doesnt work cause we use the foreground color for this, probably not a good idea to let the user change this normally yet until I think of a way to do it in a pretty way
            // ui.color_edit_button_srgba(&mut self.program_options.color_settings.text_color).on_hover_text("Text Color");
            ui.color_edit_button_srgba(
                &mut app.program_options.color_settings.info_window_color,
            )
                .on_hover_text("Info window color");
        });

        if ui.button("Reset colors to defaults").clicked() {
            app.program_options.color_settings = ColorSettings::default();
        }
    });
}
fn draw_graphing_options_menu(options_panel_ui: &mut Ui, app: &mut HappyChartState) {
    options_panel_ui.collapsing("Graphing options", |options_panel_ui| {

        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Display day lines: ");

            toggle_ui_compact(options_panel_ui, &mut app.program_options.draw_day_lines);
        });

        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Graph X Scale: ");
            options_panel_ui.add(egui::Slider::new(
                &mut app.program_options.graph_x_scale,
                0.01..=10.0,
            ))
                .on_hover_text("Multiplier used to scale the graph on the X axis.");
        });

        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Graph Y Scale: ");
            options_panel_ui.add(egui::Slider::new(
                &mut app.program_options.graph_y_scale,
                0.5..=5.0,
            ))
                .on_hover_text("Multiplier used to scale the graph on the Y axis.");
        });

        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("X Offset: ");
            options_panel_ui.add(
                egui::DragValue::new(&mut app.program_options.x_offset)
                    .speed(app.program_options.x_offset_slider_speed),
            )
                .on_hover_text("Amount of units to shift the graph on the X axis.");
        });

        // x offset slider speed
        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("X offset slider speed:");
            options_panel_ui.add(
                egui::DragValue::new(&mut app.program_options.x_offset_slider_speed)
                    .speed(0.1),
            );
        });

        // day line height
        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Day line height:");
            options_panel_ui.add(
                egui::DragValue::new(&mut app.program_options.day_line_height_offset)
                    .speed(0.1),
            );
        });
    });
}
fn draw_stat_drawing_options_menu(options_panel_ui: &mut Ui, app: &mut HappyChartState) {
    options_panel_ui.collapsing("Stat drawing options", |options_panel_ui| {

        // mouse over radius
        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Stat mouse over radius:");
            options_panel_ui.add(
                egui::DragValue::new(&mut app.program_options.mouse_over_radius)
                    .speed(0.1),
            );
        });

        // stat height offset
        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Stat height offset:");
            options_panel_ui.add(
                egui::DragValue::new(&mut app.program_options.day_stat_height_offset)
                    .speed(0.1),
            );
        });

        // day stat circle sizes
        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Stat circle radius:");
            options_panel_ui.add(
                egui::DragValue::new(&mut app.program_options.daystat_circle_size)
                    .speed(0.1),
            );
        });
        options_panel_ui.horizontal(|options_panel_ui| {
            options_panel_ui.label("Stat circle outline radius:");
            options_panel_ui.add(
                egui::DragValue::new(
                    &mut app.program_options.daystat_circle_outline_radius,
                )
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
fn draw_backup_settings_options_menu(options_panel_ui: &mut Ui, app: &mut HappyChartState, ctx: &Context) {
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

fn draw_about_page(about_page_ui: &mut Ui, app: &mut HappyChartState) {
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
        "Average sunday: {}",
        app.stats.avg_weekdays.avg_sunday
    ));
    about_page_ui.label(format!(
        "Average monday: {}",
        app.stats.avg_weekdays.avg_monday
    ));
    about_page_ui.label(format!(
        "Average tuesday: {}",
        app.stats.avg_weekdays.avg_tuesday
    ));
    about_page_ui.label(format!(
        "Average wednesday: {}",
        app.stats.avg_weekdays.avg_wednesday
    ));
    about_page_ui.label(format!(
        "Average thursday: {}",
        app.stats.avg_weekdays.avg_thursday
    ));
    about_page_ui.label(format!(
        "Average friday: {}",
        app.stats.avg_weekdays.avg_friday
    ));
    about_page_ui.label(format!(
        "Average saturday: {}",
        app.stats.avg_weekdays.avg_saturday
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

/// Update loop for egui
impl eframe::App for HappyChartState {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {

        if self.first_load {
           first_load(self,ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.input(|i| {
                for event in &i.raw.events {
                    if let egui::Event::Screenshot { viewport_id: _, image} = event {
                        handle_screenshot_event(image);
                    }
                }
            });

            let pointer_interact_pos = ctx.pointer_interact_pos();

            main_screen_button_ui(ui,self);

            click_drag_zoom_detection(ui,self,pointer_interact_pos.as_ref());

            draw_day_lines(ui,self);
            
            draw_stat_line_segments(ui,self);

            draw_stat_circles(ui,self);

            draw_stat_mouse_over_info(ui,self,ctx);

            draw_auto_update_ui(ui,self,ctx);

            draw_quit_button(ui,self,ctx);
        });

        if self.showing_options_menu {
            egui::Window::new("Options").show(ctx, |ui| {
                options_update_thread_block(ui,self);

                if ui
                    .button("Check for updates & update program")
                    .on_hover_text(self.update_status.to_text())
                    .clicked()
                {
                    self.update_thread.replace(Some(update_program()));
                }

                draw_color_options_menu(ui,self);

                draw_graphing_options_menu(ui,self);
                
                draw_stat_drawing_options_menu(ui,self);
                
                draw_backup_settings_options_menu(ui,self,ctx);
                
                if ui.button("Close Options Menu").clicked() {
                    self.showing_options_menu = false;
                }
            });
        }

        if self.showing_about_page {
            egui::Window::new("About").show(ctx, |ui| {
                draw_about_page(ui,self);
            });
        }
    }
}
