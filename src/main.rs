#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod daystat;
pub mod color_setting;

use std::fs::File;
use std::io::{Read, Write};
use eframe::egui;
use chrono::{ DateTime, Utc};
use eframe::emath::Pos2;
use egui::{Align2, Color32, FontId, Rect, Rounding, Stroke};
use std::path::Path;
use crate::daystat::daystat::DayStat;
use crate::egui::Layout;


fn main() {
    let a = DayStat{rating: 1.0, date: Utc::now().timestamp(), note: "".to_string() };
    println!("{}", a.date);
    let native_options = eframe::NativeOptions::default();

    eframe::run_native("Happy Chart", native_options, Box::new(|cc| Box::new(MyEguiApp::new(cc))));

}


#[derive(Default)]
struct MyEguiApp {
    current_time: DateTime<Utc>,
    rating: f64,
    days: Vec<DayStat>,
    first_load: bool,
    graph_xscale: f32,
    graph_yscale: f32,
    xoffset: i32,
    note_input: String,
    starting_length: usize,
    drawing_lines: bool,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self{ current_time: Default::default(), rating: 0.0, days: vec![], first_load: true, graph_xscale: 1.0, graph_yscale: 1.0, xoffset: 0, note_input: "".to_string(), starting_length: 0, drawing_lines: false }
    }
}

/// Reads the save file, if found, returns the vector full of all the DayStats
fn read_save_file() -> Vec<DayStat> {

    let path = Path::new("save.ser");

    let mut file = match File::open(path) {
        Ok(f) => {f}
        Err(_) => {
            match File::create(path){
                Ok(f) => {f}
                Err(_) => {
                    println!("couldnt create save file");
                    return vec![];
                }
            }
        }
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Ok(_) => {
            println!("successfully read save file");
        }
        Err(_) => {
            println!("unable to read save file");
            return vec![];
        }
    }
    serde_json::from_str(&s).unwrap_or_default()
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
        if self.first_load == true {
            self.first_load = false;
            self.days = read_save_file();
            self.starting_length = self.days.len();

        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.current_time = Utc::now();

            ui.horizontal(|ui| {
                ui.label("Rating: ");
                ui.add(egui::Slider::new(&mut self.rating,0.0..=100.0)).on_hover_text("The rating of the given day to be saved to the graph point.");

            });

            ui.horizontal(|ui| {
                ui.label("Graph X Scale: ");
                ui.add(egui::Slider::new(&mut self.graph_xscale, 0.1..=10.0)).on_hover_text("Multiplier used to scale the graph on the X axis.");

            });

            ui.horizontal(|ui| {
                ui.label("Graph Y Scale: ");
                ui.add(egui::Slider::new(&mut self.graph_yscale, 0.5..=5.0)).on_hover_text("Multiplier used to scale the graph on the Y axis.");

            });

            ui.horizontal(|ui| {
                ui.label("X Offset: ");
                ui.add(egui::DragValue::new(&mut self.xoffset).speed(0.1)).on_hover_text("Amount of units to shift the graph on the X axis.");

            });

            ui.horizontal(|ui| {
                ui.label("Display day lines: ");

                toggle_ui_compact(ui,&mut self.drawing_lines);

            });

            ui.horizontal(|ui| {
                ui.label("Note: ");
                ui.text_edit_singleline(&mut self.note_input).on_hover_text("The note to add to the next added graph point.");

            });

            if ui.button("Add day").clicked() {
                self.days.push(DayStat{ rating: self.rating as f32, date: self.current_time.timestamp(), note: self.note_input.clone() });
                println!("day added with rating {} and date {}", self.rating, self.current_time);
                let day = &self.days.get(self.days.len() - 1).unwrap();
                println!("{}", day);
            }

            if ui.button("Remove day").clicked() && self.days.len() > 0 {
                self.days.remove(self.days.len() - 1);
            }

            let mousepos = match ctx.pointer_hover_pos() {
                None => {Pos2::new(0.0,0.0)}
                Some(a) => {a}
            };

            //ctx.request_repaint();

            if self.drawing_lines && self.days.len() > 1 {
                // range for calculating how many lines in both directions on the x axis
                let range = {
                    if self.xoffset > 5000 { self.xoffset } else { 5000 }
                };

                for i2 in -range..range {
                    // make a fake day with the first day on the list as the first day, and add 24 hours to it each time in utc time to calculate where each line goes
                    let line_points: [Pos2; 2] = {
                        let fake_day = DayStat {
                            rating: 0.0,
                            date: self.days.get(0).unwrap().date + 86400, // 86400 = how many seconds in a day, so we are creating a fake day that starts from where the first day is
                            note: "".to_string()
                        };
                        let y: f32 = 200.0;
                        let x = {
                            let first_day = self.days.get(0).unwrap_or(&fake_day);
                            let hours: f32 = fake_day.get_hour_difference(&first_day) as f32 / 3600.0; // number of hours compared to the previous point
                            let x: f32 = (hours * self.graph_xscale) * i2 as f32;
                             x + self.xoffset as f32
                        };
                        [Pos2::new(x, y), Pos2::new(x, 800.0)]
                    };
                    ui.painter().line_segment(line_points, Stroke::new(2.0, color_setting::get_day_line_color()));
                }
            }

            let mut i = 0;
            let mut prevx = 0.0;
            let mut prevy = 0.0;

            for day in &self.days { // draw lines loop, bottom layer

                let x: f32 = calculate_x(&self.days,&day,&self.graph_xscale,&self.xoffset);

                let y: f32 = 500.0 - (day.rating * self.graph_yscale);
                let points = [Pos2::new(prevx, prevy), Pos2::new(x,y)];

                if (prevx != 0.0 && prevy != 0.0) || i == 1 { // draw line segments connecting the dots
                    ui.painter().line_segment(points,Stroke::new(2.0,color_setting::get_line_color()));
                }

                i = i + 1;
                prevx = x;
                prevy = y;
            }

            i = 0;
            for day in &self.days { // draw circles loop, middle layer

                // let x: f32 = ((i as f32 * 4.0) * self.graph_xscale) + self.xoffset as f32;
                let x: f32 = calculate_x(&self.days,&day,&self.graph_xscale,&self.xoffset);
                let y: f32 = 500.0 - (day.rating * &self.graph_yscale);

                //draw circles on each coordinate point
                ui.painter().circle_filled(Pos2::new(x, y), 4 as f32, color_setting::get_shape_color_from_rating(day.rating));

                i = i + 1;
            }

            i = 0;
            let mut moused_over = false; // boolean used to know if we are already showing mouse over text, if so, not to render it if this is true

            for day in &self.days { // draw text loop, top most layer

                let x: f32 = calculate_x(&self.days,&day,&self.graph_xscale,&self.xoffset);
                let y: f32 = 500.0 - (day.rating * self.graph_yscale);
                let rect_pos1 = Pos2::new(520.0, 10.0);
                let rect_pos2 = Pos2::new(770.0, 180.0);
                let text = day.to_string();

                let dist_max = 20.0; // maximum distance to consider a point being moused over

                if distance(&mousepos.x,&mousepos.y,&x,&y) < dist_max && moused_over == false { // draw text near by each coordinate point
                    moused_over = true;

                    ui.painter().text(Pos2::new(x + 20.0,y),Align2::LEFT_CENTER,&text,FontId::default(),color_setting::get_text_color());

                    ui.painter().rect_filled(Rect::from_two_pos(rect_pos1, rect_pos2), Rounding::from(20.0), color_setting::get_info_window_color());
                    ui.style_mut().visuals.override_text_color = Option::from(color_setting::get_text_color());

                    // info text to display in top right window
                    let mut info_text: String = day.get_date_time().to_string();
                    info_text.push_str("\n");
                    info_text.push_str(&day.rating.to_string());
                    info_text.push_str("\n");
                    info_text.push_str(&day.note);

                    ui.put(Rect::from_two_pos(rect_pos1, rect_pos2),egui::widgets::Label::new(&info_text));

                }

                i = i + 1;
            }

            ui.with_layout(Layout::bottom_up(egui::Align::BOTTOM), |ui| {

                if self.starting_length != self.days.len() {
                    ui.visuals_mut().override_text_color = Option::from(Color32::RED);
                }

                let quit_button = ui.button("Save & Quit").on_hover_text("Saves and closed the program, if text is red, changes are unsaved.");

                if quit_button.clicked() {
                    quit(frame, &self.days);
                }
            });
        });
    }
}

/// Calculates the x coordinate for each graph point
fn calculate_x(days: &Vec<DayStat>,day: &DayStat, graph_xscale: &f32, xoffset: &i32) -> f32 {

    let first_day = days.get(0).unwrap_or(&day);
    let hours: f32 = day.get_hour_difference(&first_day) as f32 / 3600.0; // number of hours compared to the previous point
    let x: f32 = (hours * graph_xscale) + *xoffset as f32;
    return x;
}

/// Returns the coordinate point distance between two points
fn distance(x1: &f32, y1: &f32, x2: &f32, y2: &f32) -> f32 {
    let g1 = (x2 - x1).powf(2.0);
    let g2 = (y2 - y1).powf(2.0);
    return (g1 + g2).sqrt();
}

/// Quit function run when the user clicks the quit button
fn quit(frame: &mut eframe::Frame, days: &Vec<DayStat>) {

    let ser = serde_json::to_string(days).unwrap();
    let deserialized: Vec<DayStat> = serde_json::from_str(&ser).unwrap();
    let path = Path::new("save.ser");

    let mut file = match File::create(path) {
        Ok(f) => {f}
        Err(_) => {panic!("unable to create save file")}
    };

    match file.write_all(ser.as_bytes()) {
        Ok(_) => {println!("successfully wrote to file!")}
        Err(_) => {println!("failed to write to file")}
    }

    println!("{}", ser);

    let mut i = 0;

    for a in deserialized {
        println!("{}: {}", i, a);
        i = i + 1;
    }

    frame.close();
}