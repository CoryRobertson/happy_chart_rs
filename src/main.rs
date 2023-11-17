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
use egui::{Align2, Color32, FontId, Rect, Rounding, Stroke, Vec2};
use self_update::{cargo_crate_version, Status};

const SAVE_FILE_NAME: &str = "save.ser";
const NEW_SAVE_FILE_NAME: &str = "happy_chart_save.ser";
const LAST_SESSION_FILE_NAME: &str = "happy_chart_last_session.ser";

const BACKUP_FILENAME_PREFIX: &str = "happy_chart_backup_";

const MANUAL_BACKUP_SUFFIX: &str = "_manual";

const BACKUP_FILE_EXTENSION: &str = "zip";

fn main() {
    let native_options = NativeOptions {
        initial_window_size: Some(read_last_session_save_file().window_size.into()),
        ..Default::default()
    };

    eframe::run_native(
        "Happy Chart",
        native_options,
        Box::new(|cc| Box::new(HappyChartState::new(cc))),
    )
    .expect("Failed to run egui app");
}

/// Update loop for egui
impl eframe::App for HappyChartState {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        // all data we need to read one time on launch, all of this most of the time is unchanging throughout usage of the program, so it can only be recalculated on launch
        // for example, day quality averages do not need to change between launches
        if self.first_load {
            self.first_load = false;
            self.days = read_save_file();

            self.days
                .sort_by(|day1, day2| day1.date.timestamp().cmp(&day2.date.timestamp()));

            self.starting_length = self.days.len();
            let ls = read_last_session_save_file();
            self.open_modulus = ls.open_modulus;
            self.program_options = ls.program_options;
            self.last_open_date = ls.last_open_date;
            self.last_backup_date = ls.last_backup_date;
            if let Some(ver) = ls.last_version_checked {
                self.auto_update_seen_version = Some(ver);
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
                                self.update_available = Some(release.clone());
                                self.update_status = AutoUpdateStatus::OutOfDate;
                            } else {
                                println!("No update available.");
                                self.update_status =
                                    AutoUpdateStatus::UpToDate(cargo_crate_version!().to_string());
                            }
                        }
                    }
                }
            }

            // check if user last backup day is +- 3 hours between the margin of their auto backup day count
            if self.program_options.auto_backup_days > -1
                && Local::now()
                    .signed_duration_since(ls.last_backup_date)
                    .num_days()
                    > i64::from(self.program_options.auto_backup_days)
            {
                backup_program_state(frame, self, false);
                self.last_backup_date = Local::now();
            }

            self.remove_old_backup_files();

            self.stats.avg_weekdays.calc_averages(&self.days);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let pointer_interact_pos = ctx.pointer_interact_pos();

            ui.horizontal(|ui| {
                ui.label("Rating: ");
                ui.add(egui::Slider::new(&mut self.rating, 0.0..=100.0))
                    .on_hover_text("The rating of the given day to be saved to the graph point.");
            });

            ui.horizontal(|ui| {
                ui.label("Note: ");
                ui.text_edit_multiline(&mut self.note_input)
                    .on_hover_text("The note to add to the next added graph point.");
            });

            if ui.button("Add day").clicked() {
                self.days.push(ImprovedDayStat {
                    rating: self.rating as f32,
                    date: ImprovedDayStat::get_current_time_system(),
                    note: self.note_input.clone(),
                });
                self.stats.avg_weekdays.calc_averages(&self.days);
                println!(
                    "day added with rating {} and date {}",
                    self.rating,
                    ImprovedDayStat::get_current_time_system()
                );
                // let day = &self.days.last().unwrap();
            }

            if ui.button("Remove day").clicked() && !self.days.is_empty() {
                self.days.remove(self.days.len() - 1);
                self.stats.avg_weekdays.calc_averages(&self.days);
            }

            ui.horizontal(|ui| {
                ui.label("Search: ");
                ui.add_sized(Vec2::new(120.0,20.0),egui::widgets::text_edit::TextEdit::singleline(&mut self.filter_term));
            });

            let mouse_pos = ctx.pointer_hover_pos().map_or_else(|| Pos2::new(0.0, 0.0), |a| a);

            // click drag and zoom detection and handling
            {
                let within_day_lines = {
                    let min_y: f32 = 220.0 - self.program_options.day_line_height_offset;
                    pointer_interact_pos.map_or(false, |pos| pos.y >= min_y)
                };

                if within_day_lines {
                    let right_click_down = ui.input(|i| i.pointer.secondary_down());

                    let left_click_down = ui.input(|i| i.pointer.primary_down());

                    // if right click is down, allow the xoffset to be moved
                    if right_click_down {
                        let drag_delta = ui.input(|i| i.pointer.delta());

                        self.program_options.x_offset += drag_delta.x;

                        // if both right click and left click are down, then we allow the x scale to be changed so the user can quickly zoom into areas on the graph
                        if left_click_down {
                            self.program_options.graph_x_scale += -drag_delta.y / 1000.0;
                            self.program_options.x_offset += drag_delta.y * (10.0);
                        }

                        if self.program_options.graph_x_scale.is_sign_negative() {
                            self.program_options.graph_x_scale = 0.001;
                        }
                    }
                }
            }

            if self.program_options.draw_day_lines && self.days.len() > 1 {
                // range for calculating how many lines in both directions on the x axis
                let range = {
                    if self.program_options.x_offset > 5000.0 {
                        self.program_options.x_offset as i32
                    } else {
                        5000
                    }
                };

                for i2 in -range..range {
                    // make a fake day with the first day on the list as the first day, and add 24 hours to it each time in utc time to calculate where each line goes
                    let line_points: [Pos2; 2] = {
                        let d = self.days.get(0).unwrap();

                        let fake_day = ImprovedDayStat {
                            rating: 0.0,
                            date: d.date.checked_add_days(Days::new(1)).unwrap_or_default(), // fake day that starts from where the first day is, with one day added
                            note: String::new(),
                        };
                        let y: f32 = 220.0 - self.program_options.day_line_height_offset;
                        let x = {
                            let first_day = d;
                            let hours: f32 =
                                fake_day.get_hour_difference(first_day) as f32 / 3600.0; // number of hours compared to the previous point

                            let x: f32 = (hours * self.program_options.graph_x_scale) * i2 as f32;
                            x + self.program_options.x_offset
                        };
                        [Pos2::new(x, y), Pos2::new(x, 800.0)]
                    };
                    ui.painter().line_segment(
                        line_points,
                        Stroke::new(2.0, self.program_options.color_settings.day_line_color),
                    );
                }
            }

            let mut i = 0;
            let mut prev_x = 0.0;
            let mut prev_y = 0.0;

            // draw lines loop, bottom layer
            if self.program_options.draw_daystat_lines {
                for day in &self.days {
                    let x: f32 = improved_calculate_x(
                        &self.days,
                        day,
                        self.program_options.graph_x_scale,
                        self.program_options.x_offset,
                    );

                    let y: f32 = day
                        .rating
                        .mul_add(-self.program_options.graph_y_scale, 500.0)
                        - self.program_options.day_stat_height_offset;
                    let points = [Pos2::new(prev_x, prev_y), Pos2::new(x, y)];

                    if (prev_x != 0.0 && prev_y != 0.0) || i == 1 {
                        // draw line segments connecting the dots
                        ui.painter().line_segment(
                            points,
                            Stroke::new(2.0, self.program_options.color_settings.line_color),
                        );
                    }

                    i += 1;
                    prev_x = x;
                    prev_y = y;
                }
            }

            i = 0;
            // draw circles loop, middle layer
            if self.program_options.draw_daystat_circles {
                for day in &self.days.clone() {
                    let x: f32 = improved_calculate_x(
                        &self.days,
                        day,
                        self.program_options.graph_x_scale,
                        self.program_options.x_offset,
                    );
                    let y: f32 = day
                        .rating
                        .mul_add(-self.program_options.graph_y_scale, 500.0)
                        - self.program_options.day_stat_height_offset;

                    //draw circles on each coordinate point
                    ui.painter().circle_filled(
                        Pos2::new(x, y),
                        self.program_options.daystat_circle_outline_radius,
                        Color32::BLACK,
                    );

                    let color = if !self.filter_term.is_empty() &&day.note.contains(&self.filter_term) {
                        Color32::LIGHT_BLUE
                    } else {
                        color_setting::get_shape_color_from_rating(day.rating)
                    };

                    ui.painter().circle_filled(
                        Pos2::new(x, y),
                        self.program_options.daystat_circle_size,
                        color,
                    );

                    i += 1;
                }
            }

            i = 0;
            let mut moused_over = false; // boolean used to know if we are already showing mouse over text, if so, not to render it if this is true

            // draw text loop, top most layer (mostly)
            for day in &self.days {
                let x: f32 = improved_calculate_x(
                    &self.days,
                    day,
                    self.program_options.graph_x_scale,
                    self.program_options.x_offset,
                );
                let y: f32 = day
                    .rating
                    .mul_add(-self.program_options.graph_y_scale, 500.0)
                    - self.program_options.day_stat_height_offset;
                let rect_pos1 = Pos2::new(520.0, 10.0);
                let rect_pos2 = Pos2::new(770.0, 180.0);
                let text = day.to_string();

                let dist_max = self.program_options.mouse_over_radius; // maximum distance to consider a point being moused over

                if distance(mouse_pos.x, mouse_pos.y, x, y) < dist_max && !moused_over {
                    // draw text near by each coordinate point
                    moused_over = true;

                    ui.painter().text(
                        Pos2::new(x + 20.0, y),
                        Align2::LEFT_CENTER,
                        &text,
                        FontId::default(),
                        self.program_options.color_settings.text_color, // color_setting::get_text_color(),
                    );

                    ui.painter().rect_filled(
                        Rect::from_two_pos(rect_pos1, rect_pos2),
                        Rounding::from(20.0),
                        self.program_options.color_settings.info_window_color,
                    );
                    ui.style_mut().visuals.override_text_color =
                        Option::from(self.program_options.color_settings.text_color);

                    // info text to display in top right window
                    let info_text: String = day.to_string();

                    ui.put(
                        Rect::from_two_pos(rect_pos1, rect_pos2),
                        egui::widgets::Label::new(&info_text),
                    );
                }
                i += 1;
            }

            // block for displaying update available message to user
            // contains dismiss update button as well
            {
                if let Some(release) = &self.update_available {
                    let should_show_update = match &self.auto_update_seen_version {
                        None => {
                            true
                        }
                        Some(ver) => {
                            self_update::version::bump_is_greater(ver,&release.version).unwrap_or(true)
                        }
                    };
                    if should_show_update {
                        if ui.button("Dismiss update").clicked() {
                            self.auto_update_seen_version = Some(release.version.to_string());
                        }
                        if ui.button("Update happy chart").clicked() {
                            self.update_thread.replace(Some(update_program()));
                            self.auto_update_seen_version = Some(release.version.to_string());
                        }
                        let mid_point_x = (ctx.screen_rect().width() / 2.0) - (250.0/2.0);
                        let quarter_point_y = ctx.screen_rect().height() / 4.0;

                        ui.painter().rect_filled(
                            Rect::from_two_pos(Pos2::new(mid_point_x, quarter_point_y), Pos2::new(mid_point_x + 250.0, quarter_point_y + 120.0)),
                            Rounding::from(4.0),
                            self.program_options.color_settings.info_window_color,
                        );
                        ui.style_mut().visuals.override_text_color =
                            Option::from(self.program_options.color_settings.text_color);

                        ui.put(
                            Rect::from_two_pos(Pos2::new(mid_point_x, quarter_point_y), Pos2::new(mid_point_x + 250.0, quarter_point_y + 120.0)),
                            egui::widgets::Label::new(format!("Update available:\n{}\nCurrent version:\nv{}\n\"Update happy chart\" to automagically update\nThis message will not display on next launch", release.name,cargo_crate_version!())),
                        );

                    }
                }
                ui.horizontal(|ui| {
                    if let Some(thread) = self.update_thread.get_mut() {
                        if !thread.is_finished() {
                            ui.label("Updating... ");
                            ui.spinner();
                        }
                    }
                });
            }

            // quit button layout
            ui.with_layout(Layout::bottom_up(egui::Align::BOTTOM), |ui| {
                if self.starting_length != self.days.len() {
                    ui.visuals_mut().override_text_color = Option::from(Color32::RED);
                } else {
                    ui.style_mut().visuals.override_text_color =
                        Option::from(self.program_options.color_settings.text_color);
                }

                ui.horizontal(|ui| {
                    let quit_button = ui.button("Save & Quit");

                    if quit_button.clicked() {
                        quit(frame, self);
                    }

                    ui.style_mut().visuals.override_text_color =
                        Option::from(self.program_options.color_settings.text_color);

                    if !self.showing_options_menu && ui.button("Options").clicked() {
                        self.showing_options_menu = true;
                    }

                    if !self.showing_about_page && ui.button("About").clicked() {
                        self.showing_about_page = true;
                    }

                    if ui.button("Save Screenshot").clicked() {
                        frame.request_screenshot();
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
        });

        if self.showing_options_menu {
            egui::Window::new("Options").show(ctx, |ui| {
                // update thread block, handles showing spinner, and checking if the update is done
                {
                    let update_thread = self.update_thread.replace(None);
                    match update_thread {
                        None => {}
                        Some(thread) => {
                            if thread.is_finished() {
                                if let Ok(res) = thread.join() { match res {
                                    Ok(status) => match status {
                                        Status::UpToDate(ver) => {
                                            self.update_status =
                                                AutoUpdateStatus::UpToDate(ver);
                                        }
                                        Status::Updated(ver) => {
                                            self.update_status = AutoUpdateStatus::Updated(ver);
                                        }
                                    },
                                    Err(err) => {
                                        self.update_status = AutoUpdateStatus::Error(err);
                                    }
                                } }
                            } else {
                                self.update_thread.replace(Some(thread));
                                self.update_status = AutoUpdateStatus::Checking;
                                ui.spinner();
                            }
                        }
                    }
                }

                if ui
                    .button("Check for updates & update program")
                    .on_hover_text(self.update_status.to_text())
                    .clicked()
                {
                    self.update_thread.replace(Some(update_program()));
                }

                // ui.horizontal(|ui| {
                //     ui.label("Update rate: ");
                //     ui.add(egui::DragValue::new(
                //         &mut self.program_options.update_modulus,
                //     ))
                //     .on_hover_text(
                //         "Automatically try to update the program every X times the program opens, -1 for disabled, 1 for every launch",
                //     );
                // });

                ui.collapsing("Color options", |ui| {
                    ui.horizontal(|ui| {
                        ui.color_edit_button_srgba(&mut self.program_options.color_settings.line_color)
                            .on_hover_text("Line color");
                        ui.color_edit_button_srgba(
                            &mut self.program_options.color_settings.day_line_color,
                        )
                            .on_hover_text("Day line color");
                        // TODO: text color doesnt work cause we use the foreground color for this, probably not a good idea to let the user change this normally yet until I think of a way to do it in a pretty way
                        // ui.color_edit_button_srgba(&mut self.program_options.color_settings.text_color).on_hover_text("Text Color");
                        ui.color_edit_button_srgba(
                            &mut self.program_options.color_settings.info_window_color,
                        )
                            .on_hover_text("Info window color");
                    });

                    if ui.button("Reset colors to defaults").clicked() {
                        self.program_options.color_settings = ColorSettings::default();
                    }
                });

                ui.collapsing("Graphing options", |ui| {

                    ui.horizontal(|ui| {
                        ui.label("Display day lines: ");

                        toggle_ui_compact(ui, &mut self.program_options.draw_day_lines);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Graph X Scale: ");
                        ui.add(egui::Slider::new(
                            &mut self.program_options.graph_x_scale,
                            0.01..=10.0,
                        ))
                            .on_hover_text("Multiplier used to scale the graph on the X axis.");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Graph Y Scale: ");
                        ui.add(egui::Slider::new(
                            &mut self.program_options.graph_y_scale,
                            0.5..=5.0,
                        ))
                            .on_hover_text("Multiplier used to scale the graph on the Y axis.");
                    });

                    ui.horizontal(|ui| {
                        ui.label("X Offset: ");
                        ui.add(
                            egui::DragValue::new(&mut self.program_options.x_offset)
                                .speed(self.program_options.x_offset_slider_speed),
                        )
                            .on_hover_text("Amount of units to shift the graph on the X axis.");
                    });

                    // x offset slider speed
                    ui.horizontal(|ui| {
                        ui.label("X offset slider speed:");
                        ui.add(
                            egui::DragValue::new(&mut self.program_options.x_offset_slider_speed)
                                .speed(0.1),
                        );
                    });

                    // day line height
                    ui.horizontal(|ui| {
                        ui.label("Day line height:");
                        ui.add(
                            egui::DragValue::new(&mut self.program_options.day_line_height_offset)
                                .speed(0.1),
                        );
                    });
                });

                ui.collapsing("Stat drawing options", |ui| {

                    // mouse over radius
                    ui.horizontal(|ui| {
                        ui.label("Stat mouse over radius:");
                        ui.add(
                            egui::DragValue::new(&mut self.program_options.mouse_over_radius)
                                .speed(0.1),
                        );
                    });

                    // stat height offset
                    ui.horizontal(|ui| {
                        ui.label("Stat height offset:");
                        ui.add(
                            egui::DragValue::new(&mut self.program_options.day_stat_height_offset)
                                .speed(0.1),
                        );
                    });

                    // day stat circle sizes
                    ui.horizontal(|ui| {
                        ui.label("Stat circle radius:");
                        ui.add(
                            egui::DragValue::new(&mut self.program_options.daystat_circle_size)
                                .speed(0.1),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Stat circle outline radius:");
                        ui.add(
                            egui::DragValue::new(
                                &mut self.program_options.daystat_circle_outline_radius,
                            )
                                .speed(0.1),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(
                            &mut self.program_options.draw_daystat_circles,
                            "Draw stat circles",
                        );
                        ui.checkbox(
                            &mut self.program_options.draw_daystat_lines,
                            "Draw stat lines",
                        );
                    });
                });

                ui.collapsing("Backup options", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Backup folder ");
                        if ui.button("Browse path").on_hover_text(format!("Current backup folder: {:?}", self.program_options.backup_save_path.clone().into_os_string())).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_directory("./")
                                .set_title("Set the location where a backup will be stored")
                                .pick_folder() {
                                self.program_options.backup_save_path = path;
                            }
                        }

                    });

                    if ui.button("Reset backup path").clicked() {
                        self.program_options.backup_save_path = ProgramOptions::default().backup_save_path;
                    }

                    ui.horizontal(|ui| {
                        ui.label("Auto backup day count: ");
                        ui.add(
                            egui::DragValue::new(&mut self.program_options.auto_backup_days)
                        ).on_hover_text("The number of days to elapse between auto backups, if less than 0, no automatic backups will take place.");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Backup age before removal: ");
                        ui.add(
                            egui::DragValue::new(&mut self.program_options.backup_age_keep_days)
                        ).on_hover_text("The number of days to elapse before deleting a backup, < 0 = never remove");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Number of stale backups before removal: ");
                        ui.add(
                            egui::DragValue::new(&mut self.program_options.number_of_kept_backups)
                        ).on_hover_text("The minimum number of stale backups needed to be present in the backups folder before the program will remove any, -1 for disabled.");
                    });

                    if ui.button("Backup program state").on_hover_text("Compress the save state and the last session data into a zip file titled with the current date.").clicked() {
                        backup_program_state(frame, self, true);
                        self.last_backup_date = Local::now();
                    }
                });

                if ui.button("Close Options Menu").clicked() {
                    self.showing_options_menu = false;
                }
            });
        }

        if self.showing_about_page {
            egui::Window::new("About").show(ctx, |ui| {
                ui.heading("Happy Chart");
                ui.label("A multi-purpose journaling software.");
                ui.separator();
                ui.label("Authors: Cory Robertson");
                ui.label("License: GPL-3.0");
                ui.horizontal(|ui| {
                    ui.label("Github repository:");
                    ui.hyperlink("https://github.com/CoryRobertson/happy_chart_rs");
                });
                ui.separator();
                ui.label(format!("Cargo crate version: {}", cargo_crate_version!()));
                ui.label(format!("Git describe: {}", GIT_DESCRIBE));
                ui.label(format!("BUILD_TIMESTAMP: {}", BUILD_TIMESTAMP));
                ui.separator();
                ui.label(format!("Day stats recorded: {}", self.days.len()));
                ui.label(format!(
                    "Average sunday: {}",
                    self.stats.avg_weekdays.avg_sunday
                ));
                ui.label(format!(
                    "Average monday: {}",
                    self.stats.avg_weekdays.avg_monday
                ));
                ui.label(format!(
                    "Average tuesday: {}",
                    self.stats.avg_weekdays.avg_tuesday
                ));
                ui.label(format!(
                    "Average wednesday: {}",
                    self.stats.avg_weekdays.avg_wednesday
                ));
                ui.label(format!(
                    "Average thursday: {}",
                    self.stats.avg_weekdays.avg_thursday
                ));
                ui.label(format!(
                    "Average friday: {}",
                    self.stats.avg_weekdays.avg_friday
                ));
                ui.label(format!(
                    "Average saturday: {}",
                    self.stats.avg_weekdays.avg_saturday
                ));
                ui.separator();
                ui.label(format!("Last backup date: {}", self.last_backup_date));
                ui.label(format!("Last open date: {}", self.last_open_date));
                ui.label(format!(
                    "Auto update seen version: {}",
                    self.auto_update_seen_version.clone().unwrap_or_default()
                ));
                ui.label(format!(
                    "Auto update status: {}",
                    &self.update_status.to_text()
                ));

                ui.separator();

                if ui.button("Close").clicked() {
                    self.showing_about_page = false;
                }
            });
        }
    }

    fn post_rendering(&mut self, window_size_px: [u32; 2], frame: &Frame) {
        if let Some(ss) = frame.screenshot() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Image", &["png", "jpeg", "jpg", "bmp", "tiff"])
                .save_file()
            {
                image::save_buffer(
                    path,
                    ss.as_raw(),
                    window_size_px[0],
                    window_size_px[1],
                    image::ColorType::Rgba8,
                )
                .unwrap();
            }
        }
    }
}
