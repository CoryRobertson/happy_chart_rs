pub mod daystat;

use std::collections::btree_map::Values;
use eframe::egui;
use chrono::{Date, DateTime, NaiveDate, NaiveDateTime, Utc};
use eframe::emath::Pos2;
use eframe::epaint::Color32;
use egui::plot::{Line, Plot};
use egui::{Align2, FontId, Stroke};
// use chrono_tz::US::Pacific;
use crate::daystat::daystat::DayStat;
use crate::egui::Layout;


fn main() {
    let a = DayStat{rating: 1.0, date: Utc::now().timestamp()};
    println!("{}", a.date);
    let native_options = eframe::NativeOptions::default();

    eframe::run_native("My egui App", native_options, Box::new(|cc| Box::new(MyEguiApp::new(cc))));

}


#[derive(Default)]
struct MyEguiApp {
    currentTime: DateTime<Utc>,
    rating: f64,
    days: Vec<DayStat>,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}


impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            self.currentTime = Utc::now();
            //frame.set_window_size(Vec2::new(300.0,300.0));



            ui.add(egui::Slider::new(&mut self.rating,0.0..=100.0));

            if ui.button("print state").clicked() {
                println!("current time: {}", self.currentTime);
                println!("rating: {}", self.rating);

                let mut count = 0;
                for day in &self.days {
                    println!("{}:{}, {}", count + 1, day.date, day.rating);
                    count = count + 1;
                }

            }

            if ui.button("add day").clicked() {
                self.days.push(DayStat{ rating: self.rating as f32, date: self.currentTime.timestamp() });
                println!("day added with rating {} and date {}", self.rating, self.currentTime);
            }

            let mousepos = match ctx.pointer_hover_pos() {
                None => {Pos2::new(0.0,0.0)}
                Some(a) => {a}
            };

            ctx.request_repaint();
            //println!("{:?}",mousepos);

            let mut i = 0;
            let mut prevx = 0.0;
            let mut prevy = 0.0;

            for day in &self.days {

                let x: f32 = i as f32 * 4.0;
                let y: f32 = 500.0 - day.rating;

                let points = [Pos2::new(prevx, prevy), Pos2::new(x,y)];

                let text = day.to_string();

                if (prevx != 0.0 && prevy != 0.0) || i == 1 {

                    ui.painter().line_segment(points,Stroke::new(8.0,Color32::from_rgb(255,0,0)));

                }

                ui.painter().circle_filled(Pos2::new(x, y), 4 as f32, Color32::from_rgb(100, 100, 100));

                ui.painter().text(Pos2::new(x + 20.0,y),Align2::LEFT_CENTER,text,FontId::default(),Color32::from_rgb(100,100,100));

                i = i + 1;
                prevx = x;
                prevy = y;

            }


            ui.with_layout(Layout::bottom_up(egui::Align::BOTTOM), |ui| {
                if ui.button("Quit").clicked() {
                    quit(frame);
                }
            });

        });
    }
}

fn quit(frame: &mut eframe::Frame) {
    frame.close();
}