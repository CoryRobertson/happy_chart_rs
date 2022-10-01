pub mod daystat;

use std::collections::btree_map::Values;
use std::fs::File;
use std::io::{Read, Write};
use eframe::egui;
use chrono::{Date, DateTime, NaiveDate, NaiveDateTime, Utc};
use eframe::emath::Pos2;
use eframe::epaint::Color32;
use egui::plot::{Line, Plot};
use egui::{Align2, FontId, Stroke};
use serde::{Serialize, Serializer};
use serde::ser::SerializeSeq;
use std::path::Path;

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
    firstLoad: bool,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.


        Self{ currentTime: Default::default(), rating: 0.0, days: vec![], firstLoad: true}
        // Self::default()
    }
}

fn readSaveFile() -> Vec<DayStat> {
    let path = Path::new("save.ser");

    let mut file = match File::open(path) {
        Ok(f) => {f}
        Err(_) => {
            match File::create(path){
                Ok(f) => {f}
                Err(_) => {panic!("couldnt create save file")}
            }
        }
    };


    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Ok(_) => {
            println!("successfuly read save file");
        }
        Err(_) => {
            println!("unable to read save file");
            return vec![];
        }
    }
    serde_json::from_str(&s).unwrap_or_default()
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        if self.firstLoad == true {
            self.firstLoad = false;
            self.days = readSaveFile();
        }

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
                let day = &self.days.get(self.days.len() - 1).unwrap();
                println!("{}", day);
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

                    quit(frame, &self.days);
                }
            });

        });
    }
}


fn quit(frame: &mut eframe::Frame, days: &Vec<DayStat>) {

    let ser = serde_json::to_string(days).unwrap();
    let deserialized: Vec<DayStat> = serde_json::from_str(&ser).unwrap();
    let path = Path::new("save.ser");

    let mut file = match File::create(path) {
        Ok(f) => {f}
        Err(_) => {panic!("unable to create save file")}
    };

    match file.write_all(ser.as_bytes()) {
        Ok(_) => {println!("successfuly wrote to file!")}
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