pub mod daystat;

use std::fs::File;
use std::io::{Read, Write};
use eframe::egui;
use chrono::{ DateTime, Utc};
use eframe::emath::Pos2;
use eframe::epaint::Color32;
use egui::{Align2, FontId, Stroke};
use std::path::Path;

// use chrono_tz::US::Pacific;
use crate::daystat::daystat::DayStat;
use crate::egui::Layout;


fn main() {
    let a = DayStat{rating: 1.0, date: Utc::now().timestamp()};
    println!("{}", a.date);
    let native_options = eframe::NativeOptions::default();

    eframe::run_native("happy chart", native_options, Box::new(|cc| Box::new(MyEguiApp::new(cc))));

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
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.


        Self{ current_time: Default::default(), rating: 0.0, days: vec![], first_load: true, graph_xscale: 1.0, graph_yscale: 1.0, xoffset: 0 }
        // Self::default()
    }
}

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

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        if self.first_load == true {
            self.first_load = false;
            self.days = read_save_file();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("Hello World!");



            self.current_time = Utc::now();
            //frame.set_window_size(Vec2::new(300.0,300.0));




            ui.horizontal(|ui| {
                ui.label("Rating: ");
                ui.add(egui::Slider::new(&mut self.rating,0.0..=100.0));

            });

            ui.horizontal(|ui| {
                ui.label("Graph X Scale: ");
                ui.add(egui::Slider::new(&mut self.graph_xscale, 0.1..=10.0));

            });

            ui.horizontal(|ui| {
                ui.label("Graph Y Scale: ");
                ui.add(egui::Slider::new(&mut self.graph_yscale, 0.5..=10.0));

            });

            // if ui.button("print state").clicked() {
            //     println!("current time: {}", self.current_time);
            //     println!("rating: {}", self.rating);
            //
            //     let mut count = 0;
            //     for day in &self.days {
            //         println!("{}:{}, {}", count + 1, day.date, day.rating);
            //         count = count + 1;
            //     }
            //
            // }

            ui.horizontal(|ui| {

                if ui.button("shift left").clicked() {
                    self.xoffset = self.xoffset - 10;
                }

                if ui.button("shift right").clicked() {
                    self.xoffset = self.xoffset + 10;
                }
            });



            if ui.button("add day").clicked() {
                self.days.push(DayStat{ rating: self.rating as f32, date: self.current_time.timestamp() });
                println!("day added with rating {} and date {}", self.rating, self.current_time);
                let day = &self.days.get(self.days.len() - 1).unwrap();
                println!("{}", day);
            }

            if ui.button("remove day").clicked() {
                self.days.remove(self.days.len() - 1);
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

                let x: f32 = ((i as f32 * 4.0) * self.graph_xscale) + self.xoffset as f32;
                let y: f32 = (500.0 - (day.rating * self.graph_yscale));

                let points = [Pos2::new(prevx, prevy), Pos2::new(x,y)];

                let text = day.to_string();

                if (prevx != 0.0 && prevy != 0.0) || i == 1 {

                    ui.painter().line_segment(points,Stroke::new(8.0,Color32::from_rgb(255,0,0)));

                }

                ui.painter().circle_filled(Pos2::new(x, y), 4 as f32, Color32::from_rgb(100, 100, 100));

                // TODO: make text only show up when mouse cursor is somewhat close to it for readability purposes
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