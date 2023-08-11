#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod color_setting;
mod daystat;
mod improved_daystat;
mod last_session;
mod program_options;

const GIT_DESCRIBE: &str = env!("VERGEN_GIT_DESCRIBE");
const BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

#[allow(deprecated)]
use crate::daystat::DayStat;
use crate::egui::Layout;
use crate::improved_daystat::ImprovedDayStat;
use crate::last_session::LastSession;
use crate::program_options::ProgramOptions;
use chrono::{DateTime, Days, Local, Utc};
use eframe::emath::Pos2;
use eframe::{egui, NativeOptions};
use egui::{Align2, Color32, FontId, Rect, Rounding, Stroke};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const SAVE_FILE_NAME: &str = "save.ser";
const NEW_SAVE_FILE_NAME: &str = "happy_chart_save.ser";
const LAST_SESSION_FILE_NAME: &str = "happy_chart_last_session.ser";

fn main() {
    let native_options = NativeOptions {
        initial_window_size: Some(read_last_session_save_file().window_size.into()),
        ..Default::default()
    };

    eframe::run_native(
        "Happy Chart",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc))),
    )
    .expect("Failed to run egui app");
}

#[derive(Default)]
struct MyEguiApp {
    current_time: DateTime<Utc>,
    rating: f64,
    days: Vec<ImprovedDayStat>,
    first_load: bool,
    graph_x_scale: f32,
    graph_y_scale: f32,
    x_offset: i32,
    note_input: String,
    starting_length: usize,
    drawing_lines: bool,
    showing_options_menu: bool,
    program_options: ProgramOptions,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_time: Default::default(),
            rating: 0.0,
            days: vec![],
            first_load: true,
            graph_x_scale: 1.0,
            graph_y_scale: 1.0,
            x_offset: 0,
            note_input: "".to_string(),
            starting_length: 0,
            drawing_lines: false,
            showing_options_menu: false,
            program_options: ProgramOptions::default(),
        }
    }
}
/// Reads the last session file, if exists, returns the deserialized contents, if it doesnt exist, returns a default LastSession struct.
fn read_last_session_save_file() -> LastSession {
    let path = Path::new(LAST_SESSION_FILE_NAME);

    let mut file = match File::open(path) {
        // try to open save file
        Ok(f) => f,
        Err(_) => {
            match File::create(path) {
                // save file wasn't found, make one
                Ok(f) => {
                    println!("last session save file not found, creating one");
                    f
                }
                Err(_) => {
                    // cant make save file, return a default last session just encase
                    return LastSession::default();
                }
            }
        }
    };
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        // try to read the file into a string
        Ok(_) => {
            println!("read last session save file successfully");
        }
        Err(_) => {
            // fail to read file as string, return a default last session just encase, this should only happen if invalid utf-8 exists in the save file.
            return LastSession::default();
        }
    }
    serde_json::from_str(&s).unwrap_or_default() // return the deserialized struct
}

/// Reads the save file, if found, returns the vector full of all the DayStats
fn read_save_file() -> Vec<ImprovedDayStat> {
    let new_path = PathBuf::from(NEW_SAVE_FILE_NAME);
    let path = Path::new(SAVE_FILE_NAME);

    let mut file = match File::open(&new_path) {
        Ok(f) => f,
        Err(_) => match File::open(path) {
            Ok(f) => f,
            Err(_) => match File::create(new_path) {
                Ok(f) => f,
                Err(err) => {
                    panic!("cant create new save file: {}", err);
                }
            },
        },
    };

    let mut s = String::new();
    let read_len = match file.read_to_string(&mut s) {
        Ok(read_len) => {
            println!("successfully read save file");
            read_len
        }
        Err(_) => {
            println!("unable to read save file");
            return vec![];
        }
    };

    // attempt to read old save file format
    match serde_json::from_str::<Vec<ImprovedDayStat>>(&s[0..read_len]) {
        Ok(vec) => {
            println!("found modern save file");
            // new save file format found, return it
            vec
        }
        Err(_) => {
            // not old save file format, attempt to read it as new save file format
            #[allow(deprecated)]
            match serde_json::from_str::<Vec<DayStat>>(&s[0..read_len]) {
                Ok(v) => {
                    println!("found legacy save file, converting");
                    // old save file format found, convert it into new save file format
                    v.into_iter()
                        .map(|old_day_stat| old_day_stat.into())
                        .collect::<Vec<ImprovedDayStat>>()
                }
                Err(_) => {
                    // cant read old or new save file format, so empty vec.
                    vec![]
                }
            }
        }
    }
}

// thank you online example <3
fn toggle_ui_compact(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
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

/// Update loop for egui
impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // load previous save on first load in
        if self.first_load {
            self.first_load = false;
            self.days = read_save_file();
            self.starting_length = self.days.len();
            let ls = read_last_session_save_file();
            self.x_offset = ls.xoffset;
            self.graph_x_scale = ls.graph_xscale;
            self.graph_y_scale = ls.graph_yscale;
            self.drawing_lines = ls.displaying_day_lines;
            self.program_options = ls.program_options;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.current_time = Utc::now();

            ui.horizontal(|ui| {
                ui.label("Rating: ");
                ui.add(egui::Slider::new(&mut self.rating, 0.0..=100.0))
                    .on_hover_text("The rating of the given day to be saved to the graph point.");
            });

            ui.horizontal(|ui| {
                ui.label("Graph X Scale: ");
                ui.add(egui::Slider::new(&mut self.graph_x_scale, 0.01..=10.0))
                    .on_hover_text("Multiplier used to scale the graph on the X axis.");
            });

            ui.horizontal(|ui| {
                ui.label("Graph Y Scale: ");
                ui.add(egui::Slider::new(&mut self.graph_y_scale, 0.5..=5.0))
                    .on_hover_text("Multiplier used to scale the graph on the Y axis.");
            });

            ui.horizontal(|ui| {
                ui.label("X Offset: ");
                ui.add(
                    egui::DragValue::new(&mut self.x_offset)
                        .speed(self.program_options.x_offset_slider_speed),
                )
                .on_hover_text("Amount of units to shift the graph on the X axis.");
            });

            ui.horizontal(|ui| {
                ui.label("Display day lines: ");

                toggle_ui_compact(ui, &mut self.drawing_lines);
            });

            ui.horizontal(|ui| {
                ui.label("Note: ");
                ui.text_edit_multiline(&mut self.note_input)
                    .on_hover_text("The note to add to the next added graph point.");
            });

            if ui.button("Add day").clicked() {
                self.days.push(ImprovedDayStat {
                    rating: self.rating as f32,
                    date: self.current_time.with_timezone(&Local),
                    note: self.note_input.clone(),
                });
                println!(
                    "day added with rating {} and date {}",
                    self.rating, self.current_time
                );
                // let day = &self.days.last().unwrap();
            }

            if ui.button("Remove day").clicked() && !self.days.is_empty() {
                self.days.remove(self.days.len() - 1);
            }

            let mouse_pos = match ctx.pointer_hover_pos() {
                None => Pos2::new(0.0, 0.0),
                Some(a) => a,
            };

            //ctx.request_repaint();

            if self.drawing_lines && self.days.len() > 1 {
                // range for calculating how many lines in both directions on the x axis
                let range = {
                    if self.x_offset > 5000 {
                        self.x_offset
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
                            note: "".to_string(),
                        };
                        let y: f32 = 220.0 - self.program_options.day_line_height_offset;
                        let x = {
                            let first_day = d;
                            let hours: f32 =
                                fake_day.get_hour_difference(first_day) as f32 / 3600.0; // number of hours compared to the previous point

                            let x: f32 = (hours * self.graph_x_scale) * i2 as f32;
                            x + self.x_offset as f32
                        };
                        [Pos2::new(x, y), Pos2::new(x, 800.0)]
                    };
                    ui.painter().line_segment(
                        line_points,
                        Stroke::new(2.0, color_setting::get_day_line_color()),
                    );
                }
            }

            let mut i = 0;
            let mut prev_x = 0.0;
            let mut prev_y = 0.0;

            // draw lines loop, bottom layer
            for day in &self.days {
                let x: f32 =
                    improved_calculate_x(&self.days, day, &self.graph_x_scale, &self.x_offset);

                let y: f32 = (500.0 - (day.rating * self.graph_y_scale))
                    - self.program_options.day_stat_height_offset;
                let points = [Pos2::new(prev_x, prev_y), Pos2::new(x, y)];

                if (prev_x != 0.0 && prev_y != 0.0) || i == 1 {
                    // draw line segments connecting the dots
                    ui.painter()
                        .line_segment(points, Stroke::new(2.0, color_setting::get_line_color()));
                }

                i += 1;
                prev_x = x;
                prev_y = y;
            }

            i = 0;
            // draw circles loop, middle layer
            for day in &self.days.clone() {
                let x: f32 =
                    improved_calculate_x(&self.days, day, &self.graph_x_scale, &self.x_offset);
                let y: f32 = (500.0 - (day.rating * self.graph_y_scale))
                    - self.program_options.day_stat_height_offset;

                //draw circles on each coordinate point
                ui.painter().circle_filled(
                    Pos2::new(x, y),
                    4_f32,
                    color_setting::get_shape_color_from_rating(day.rating),
                );

                i += 1;
            }

            i = 0;
            let mut moused_over = false; // boolean used to know if we are already showing mouse over text, if so, not to render it if this is true

            // draw text loop, top most layer
            for day in &self.days {
                let x: f32 =
                    improved_calculate_x(&self.days, day, &self.graph_x_scale, &self.x_offset);
                let y: f32 = (500.0 - (day.rating * self.graph_y_scale)) - self.program_options.day_stat_height_offset;
                let rect_pos1 = Pos2::new(520.0, 10.0);
                let rect_pos2 = Pos2::new(770.0, 180.0);
                let text = day.to_string();

                let dist_max = self.program_options.mouse_over_radius; // maximum distance to consider a point being moused over

                if distance(&mouse_pos.x, &mouse_pos.y, &x, &y) < dist_max && !moused_over {
                    // draw text near by each coordinate point
                    moused_over = true;

                    ui.painter().text(
                        Pos2::new(x + 20.0, y),
                        Align2::LEFT_CENTER,
                        &text,
                        FontId::default(),
                        color_setting::get_text_color(),
                    );

                    ui.painter().rect_filled(
                        Rect::from_two_pos(rect_pos1, rect_pos2),
                        Rounding::from(20.0),
                        color_setting::get_info_window_color(),
                    );
                    ui.style_mut().visuals.override_text_color =
                        Option::from(color_setting::get_text_color());

                    // info text to display in top right window
                    let info_text: String = day.to_string();

                    ui.put(
                        Rect::from_two_pos(rect_pos1, rect_pos2),
                        egui::widgets::Label::new(&info_text),
                    );
                }
                i += 1;
            }

            // quit button layout
            ui.with_layout(Layout::bottom_up(egui::Align::BOTTOM), |ui| {
                if self.starting_length != self.days.len() {
                    ui.visuals_mut().override_text_color = Option::from(Color32::RED);
                } else {
                    ui.style_mut().visuals.override_text_color =
                        Option::from(color_setting::get_text_color());
                }

                let quit_button = ui.button("Save & Quit");

                if quit_button.clicked() {
                    quit(frame, self);
                }

                if !self.showing_options_menu && ui.button("Options").clicked() {
                    self.showing_options_menu = true;
                }

                if quit_button.hovered() {
                    ui.label(
                        egui::RichText::new(BUILD_TIMESTAMP).color(Color32::from_rgb(80, 80, 80)),
                    );
                    ui.label(
                        egui::RichText::new(GIT_DESCRIBE).color(Color32::from_rgb(80, 80, 80)),
                    );
                }
            });
        });

        if self.showing_options_menu {
            egui::Window::new("Options").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("X offset slider speed:");
                    ui.add(
                        egui::DragValue::new(&mut self.program_options.x_offset_slider_speed)
                            .speed(0.1),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Stat height offset:");
                    ui.add(
                        egui::DragValue::new(&mut self.program_options.day_stat_height_offset)
                            .speed(0.1),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Day line height:");
                    ui.add(
                        egui::DragValue::new(&mut self.program_options.day_line_height_offset)
                            .speed(0.1),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Stat mouse over radius:");
                    ui.add(
                        egui::DragValue::new(&mut self.program_options.mouse_over_radius)
                            .speed(0.1),
                    );
                });

                if ui.button("Close Options Menu").clicked() {
                    self.showing_options_menu = false;
                }
            });
        }
    }
}

/// Calculates the x coordinate for each graph point
#[deprecated]
#[allow(dead_code, deprecated)]
fn calculate_x(days: &[DayStat], day: &DayStat, graph_xscale: &f32, xoffset: &i32) -> f32 {
    let first_day = days.get(0).unwrap_or(day);
    let hours: f32 = day.get_hour_difference(first_day) as f32 / 3600.0; // number of hours compared to the previous point
    let x: f32 = (hours * graph_xscale) + *xoffset as f32;
    x
}

/// Calculates the x coordinate for each graph point
fn improved_calculate_x(
    days: &[ImprovedDayStat],
    day: &ImprovedDayStat,
    graph_x_scale: &f32,
    x_offset: &i32,
) -> f32 {
    let first_day = days.get(0).unwrap_or(day);
    let hours: f32 = day.get_hour_difference(first_day) as f32 / 3600.0; // number of hours compared to the previous point
    let x: f32 = (hours * graph_x_scale) + *x_offset as f32;
    x
}

/// Returns the coordinate point distance between two points
fn distance(x1: &f32, y1: &f32, x2: &f32, y2: &f32) -> f32 {
    let g1 = (x2 - x1).powf(2.0);
    let g2 = (y2 - y1).powf(2.0);
    (g1 + g2).sqrt()
}

/// Quit function run when the user clicks the quit button
fn quit(frame: &mut eframe::Frame, app: &MyEguiApp) {
    let days = &app.days;

    let last_session = LastSession {
        graph_xscale: app.graph_x_scale,
        graph_yscale: app.graph_y_scale,
        xoffset: app.x_offset,
        displaying_day_lines: app.drawing_lines,
        window_size: frame.info().window_info.size.into(),
        program_options: app.program_options.clone(),
    };

    let session_ser = serde_json::to_string(&last_session).unwrap();
    let last_session_path = Path::new(LAST_SESSION_FILE_NAME);

    let mut last_session_save_file = match File::create(last_session_path) {
        Ok(f) => f,
        Err(_) => {
            panic!("unable to create last_session_save_file")
        }
    };

    match last_session_save_file.write_all(session_ser.as_bytes()) {
        Ok(_) => {
            println!("successfully wrote to last_session_save")
        }
        Err(_) => {
            println!("failed to write to last_session_save")
        }
    }

    let ser = serde_json::to_string(days).unwrap();
    let save_path = Path::new(NEW_SAVE_FILE_NAME);

    let mut save_file = match File::create(save_path) {
        Ok(f) => f,
        Err(_) => {
            panic!(
                "unable to create save {:?}",
                save_path.file_name().unwrap_or_default()
            )
        }
    };

    match save_file.write_all(ser.as_bytes()) {
        Ok(_) => {
            println!(
                "successfully wrote to {:?}!",
                save_path.file_name().unwrap_or_default()
            )
        }
        Err(_) => {
            println!(
                "failed to write to {:?}",
                save_path.file_name().unwrap_or_default()
            )
        }
    }
    frame.close();
}
