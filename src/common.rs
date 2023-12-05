#[allow(deprecated)]
use crate::daystat::DayStat;
use crate::happy_chart_state::HappyChartState;
use crate::improved_daystat::ImprovedDayStat;
use crate::last_session::LastSession;
use crate::{
    BACKUP_FILENAME_PREFIX, BACKUP_FILE_EXTENSION, LAST_SESSION_FILE_NAME, MANUAL_BACKUP_SUFFIX,
    NEW_SAVE_FILE_NAME, SAVE_FILE_NAME,
};
use chrono::{DateTime, Datelike, Local, Weekday};
use eframe::egui;
use egui::{Pos2, Rect, ViewportCommand};
use self_update::update::Release;
use self_update::{cargo_crate_version, Status};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::{fs, thread};
use zip::write::FileOptions;
use zip::CompressionMethod;

/// Calculates the x coordinate for each graph point
#[deprecated]
#[allow(dead_code, deprecated)]
fn calculate_x(days: &[DayStat], day: &DayStat, graph_xscale: f32, xoffset: i32) -> f32 {
    let first_day = days.get(0).unwrap_or(day);
    let hours: f32 = day.get_hour_difference(first_day) as f32 / 3600.0; // number of hours compared to the previous point
    let x: f32 = hours.mul_add(graph_xscale, xoffset as f32);
    x
}

/// Calculates the x coordinate for each graph point
pub fn improved_calculate_x(
    days: &[ImprovedDayStat],
    day: &ImprovedDayStat,
    graph_x_scale: f32,
    x_offset: f32,
) -> f32 {
    let first_day = days.get(0).unwrap_or(day);
    let hours: f32 = day.get_hour_difference(first_day) as f32 / 3600.0; // number of hours compared to the previous point
    let x: f32 = hours.mul_add(graph_x_scale, x_offset);
    x
}

/// Returns the coordinate point distance between two points
pub fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let g1 = (x2 - x1).powi(2);
    let g2 = (y2 - y1).powi(2);
    (g1 + g2).sqrt()
}

/// Quit function run when the user clicks the quit button
pub fn quit(ctx: &egui::Context, app: &HappyChartState) {
    save_program_state(ctx, app);

    ctx.send_viewport_cmd(ViewportCommand::Close);
    // frame.close();
}

fn get_backup_file_name(time: &DateTime<Local>, is_manual: bool) -> String {
    format!(
        "{}{}-{}-{}{}.{}",
        BACKUP_FILENAME_PREFIX,
        time.month(),
        time.day(),
        time.year(),
        {
            if is_manual {
                MANUAL_BACKUP_SUFFIX
            } else {
                ""
            }
        },
        BACKUP_FILE_EXTENSION
    )
}

pub fn backup_program_state(ctx: &egui::Context, app: &HappyChartState, is_manual: bool) {
    let time = Local::now();
    save_program_state(ctx, app);
    let _ = fs::create_dir_all(&app.program_options.backup_save_path);
    let archive_file_name = get_backup_file_name(&time, is_manual);
    let file = File::create(
        app.program_options
            .backup_save_path
            .clone()
            .join(Path::new(&archive_file_name)),
    );
    let mut arch = zip::ZipWriter::new(file.unwrap());
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
    if let Ok(mut old_save_file) = File::open(SAVE_FILE_NAME) {
        let _ = arch.start_file(SAVE_FILE_NAME, options);
        let mut old_file_bytes = vec![];
        let _ = old_save_file.read_to_end(&mut old_file_bytes);
        let _ = arch.write_all(&old_file_bytes);
    } else {
        // no old save file present, so we can just
    }
    let mut new_save_file = File::open(NEW_SAVE_FILE_NAME).unwrap();
    let mut last_session_file = File::open(LAST_SESSION_FILE_NAME).unwrap();
    let _ = arch.start_file(NEW_SAVE_FILE_NAME, options);
    let mut new_file_bytes = vec![];
    let _ = new_save_file.read_to_end(&mut new_file_bytes);
    let _ = arch.write_all(&new_file_bytes);
    let _ = arch.start_file(LAST_SESSION_FILE_NAME, options);
    let mut last_session_file_bytes = vec![];
    let _ = last_session_file.read_to_end(&mut last_session_file_bytes);
    let _ = arch.write_all(&last_session_file_bytes);
    let _ = arch.finish();
}

pub fn save_program_state(ctx: &egui::Context, app: &HappyChartState) {
    let days = &app.days;

    let window_size = ctx.input(|i| {
        i.viewport().inner_rect.unwrap_or(Rect::from_two_pos(
            Pos2::new(0.0, 0.0),
            Pos2::new(600.0, 600.0),
        ))
    });

    let last_session = LastSession {
        window_size: [window_size.width(), window_size.height()],
        program_options: app.program_options.clone(),
        open_modulus: app.open_modulus + 1,
        last_open_date: Local::now(),
        last_version_checked: {
            app.auto_update_seen_version
                .as_ref()
                .map(std::string::ToString::to_string)
        },
        last_backup_date: app.last_backup_date,
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
            println!("successfully wrote to last_session_save");
        }
        Err(_) => {
            println!("failed to write to last_session_save");
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
            );
        }
        Err(_) => {
            println!(
                "failed to write to {:?}",
                save_path.file_name().unwrap_or_default()
            );
        }
    }
}

pub fn get_average_for_day_of_week(day_of_week: Weekday, days: &[ImprovedDayStat]) -> f32 {
    let ratings = days
        .iter()
        .filter(|stat| stat.date.weekday() == day_of_week)
        .map(|stat| stat.rating)
        .collect::<Vec<f32>>();

    ratings.iter().sum::<f32>() / ratings.len() as f32
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

pub fn get_release_list() -> Result<Vec<Release>, Box<dyn std::error::Error>> {
    let list = self_update::backends::github::ReleaseList::configure()
        .repo_owner("CoryRobertson")
        .repo_name("happy_chart_rs")
        .build()?
        .fetch()?;
    #[cfg(debug_assertions)]
    println!("{:?}", list);
    Ok(list)
}

/// Reads the last session file, if exists, returns the deserialized contents, if it doesnt exist, returns a default `LastSession` struct.
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

/// Reads the save file, if found, returns the vector full of all the `DayStats`
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
