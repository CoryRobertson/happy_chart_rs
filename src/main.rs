#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
        // load previous save on first load in
        if self.first_load == true {
            self.first_load = false;
            self.days = read_save_file();
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            self.current_time = Utc::now();

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

            ui.horizontal(|ui| {
                if ui.button("shift left").clicked() {
                    self.xoffset = self.xoffset - 10;
                }
                if ui.button("shift right").clicked() {
                    self.xoffset = self.xoffset + 10;
                }
            });

            ui.horizontal(|ui| {
                ui.label("X Offset: ");
                ui.add(egui::DragValue::new(&mut self.xoffset).speed(0.1));

            });

            if ui.button("add day").clicked() {
                self.days.push(DayStat{ rating: self.rating as f32, date: self.current_time.timestamp() });
                println!("day added with rating {} and date {}", self.rating, self.current_time);
                let day = &self.days.get(self.days.len() - 1).unwrap();
                println!("{}", day);
            }

            if ui.button("add sine wave").clicked() {
                for a in 0..1000 {
                    let special_rating = (((a as f32 / 100.0).sin() * 100.0) + 100.0) / 2.0;
                    self.days.push(DayStat { rating: special_rating, date: self.current_time.timestamp() });
                }
            }

            if ui.button("remove day").clicked() {
                self.days.remove(self.days.len() - 1);
            }

            let mousepos = match ctx.pointer_hover_pos() {
                None => {Pos2::new(0.0,0.0)}
                Some(a) => {a}
            };

            ctx.request_repaint();

            let mut i = 0;
            let mut prevx = 0.0;
            let mut prevy = 0.0;

            for day in &self.days { // draw lines loop, bottom layer

                let x: f32 = ((i as f32 * 4.0) * self.graph_xscale) + self.xoffset as f32;
                let y: f32 = 500.0 - (day.rating * self.graph_yscale);

                let points = [Pos2::new(prevx, prevy), Pos2::new(x,y)];

                let segment_color = Color32::from_rgb(100,100,100);

                if (prevx != 0.0 && prevy != 0.0) || i == 1 { // draw line segments connecting the dots
                    ui.painter().line_segment(points,Stroke::new(2.0,segment_color));
                }

                i = i + 1;

                prevx = x;
                prevy = y;

            }

            i = 0;
            for day in &self.days { // draw circles loop, middle layer

                let x: f32 = ((i as f32 * 4.0) * self.graph_xscale) + self.xoffset as f32;
                let y: f32 = 500.0 - (day.rating * self.graph_yscale);

                let circle_color = get_shape_color_from_rating(day.rating);

                //draw circles on each coordinate point

                ui.painter().circle_filled(Pos2::new(x, y), 4 as f32, circle_color);

                i = i + 1;

            }

            i = 0;
            let mut moused_over = false; // boolean used to know if we are already showing mouse over text, if so, not to render it if this is true

            for day in &self.days { // draw text loop, top most layer

                let x: f32 = ((i as f32 * 4.0) * self.graph_xscale) + self.xoffset as f32;
                let y: f32 = 500.0 - (day.rating * self.graph_yscale);

                let text = day.to_string();

                let text_color = Color32::from_rgb(255,255,255);

                let dist_max = ((5.0*self.graph_xscale) + (5.0*self.graph_yscale)) / 2.0;

                if distance(&mousepos.x,&mousepos.y,&x,&y) < dist_max && moused_over == false { // draw text near by each coordinate point
                    ui.painter().text(Pos2::new(x + 20.0,y),Align2::LEFT_CENTER,text,FontId::default(),text_color);
                    moused_over = true;
                }

                i = i + 1;

            }

            ui.with_layout(Layout::bottom_up(egui::Align::BOTTOM), |ui| {
                if ui.button("Quit").clicked() {
                    quit(frame, &self.days);
                }
            });

        });
    }
}

fn get_shape_color_from_rating(rating: f32) -> Color32 {

    let new_rating = rating / 100.0;

    let red: u8 = (100.0/new_rating) as u8;
    let green: u8 = (new_rating * 255.0) as u8;
    let blue: u8 = (new_rating * 50.0) as u8;

    Color32::from_rgb(red,green,blue)
}

fn distance(x1: &f32, y1: &f32, x2: &f32, y2: &f32) -> f32 {
    let g1 = (x2 - x1).powf(2.0);
    let g2 = (y2 - y1).powf(2.0);
    return (g1 + g2).sqrt();
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