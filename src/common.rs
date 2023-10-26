#[allow(deprecated)]
use crate::daystat::DayStat;
use crate::happy_chart_state::HappyChartState;
use crate::improved_daystat::ImprovedDayStat;
use crate::last_session::LastSession;
use crate::{LAST_SESSION_FILE_NAME, NEW_SAVE_FILE_NAME, SAVE_FILE_NAME};
use eframe::{egui, Frame};
use self_update::{cargo_crate_version, Status};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::thread::JoinHandle;

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
pub fn improved_calculate_x(
    days: &[ImprovedDayStat],
    day: &ImprovedDayStat,
    graph_x_scale: &f32,
    x_offset: &f32,
) -> f32 {
    let first_day = days.get(0).unwrap_or(day);
    let hours: f32 = day.get_hour_difference(first_day) as f32 / 3600.0; // number of hours compared to the previous point
    let x: f32 = (hours * graph_x_scale) + *x_offset;
    x
}

/// Returns the coordinate point distance between two points
pub fn distance(x1: &f32, y1: &f32, x2: &f32, y2: &f32) -> f32 {
    let g1 = (x2 - x1).powi(2);
    let g2 = (y2 - y1).powi(2);
    (g1 + g2).sqrt()
}

/// Quit function run when the user clicks the quit button
pub fn quit(frame: &mut Frame, app: &HappyChartState) {
    let days = &app.days;

    let last_session = LastSession {
        window_size: frame.info().window_info.size.into(),
        program_options: app.program_options.clone(),
        open_modulus: app.open_modulus + 1,
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

pub fn update_program() -> JoinHandle<Result<Status, String>> {
    thread::spawn(|| {
        match self_update::backends::github::UpdateBuilder::new()
            .repo_owner("CoryRobertson")
            .repo_name("happy_chart_rs")
            .bin_name("happy_chart_rs")
            .show_download_progress(true)
            .no_confirm(true)
            .current_version(cargo_crate_version!())
            .build()
        {
            Ok(updater) => match updater.update() {
                Ok(status) => Ok(status),
                Err(err) => Err(err.to_string()),
            },
            Err(err) => Err(err.to_string()),
        }
    })
}

/// Reads the last session file, if exists, returns the deserialized contents, if it doesnt exist, returns a default LastSession struct.
pub fn read_last_session_save_file() -> LastSession {
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
pub fn read_save_file() -> Vec<ImprovedDayStat> {
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
