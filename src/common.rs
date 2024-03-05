use crate::auto_update_status::AutoUpdateStatus;
#[allow(deprecated)]
use crate::day_stats::daystat::DayStat;
use crate::day_stats::improved_daystat::ImprovedDayStat;
use crate::last_session::LastSession;
use crate::state::error_states::HappyChartError;
use crate::state::happy_chart_state::HappyChartState;
use crate::{
    BACKUP_FILENAME_PREFIX, BACKUP_FILE_EXTENSION, LAST_SESSION_FILE_NAME, MANUAL_BACKUP_SUFFIX,
    NEW_SAVE_FILE_NAME, SAVE_FILE_NAME,
};
use chrono::{DateTime, Datelike, Local, Weekday};
use eframe::egui;
use eframe::epaint::ColorImage;
use egui::{Color32, Context, Pos2, Rect, Ui, ViewportCommand};
use self_update::update::Release;
use self_update::{cargo_crate_version, Status};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::SystemTime;
use std::{fs, thread};
use zip::write::FileOptions;
use zip::CompressionMethod;

/// Calculates the x coordinate for each graph point
#[tracing::instrument]
pub fn improved_calculate_x(
    days: &[ImprovedDayStat],
    day: &ImprovedDayStat,
    graph_x_scale: f32,
    x_offset: f32,
) -> f32 {
    let first_day = days.first().unwrap_or(day);
    let hours: f32 = day.get_hour_difference(first_day) as f32 / 3600.0; // number of hours compared to the previous point
    let x: f32 = hours.mul_add(graph_x_scale, x_offset);
    x
}

/// Returns the coordinate point distance between two points
#[tracing::instrument]
pub fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let g1 = (x2 - x1).powi(2);
    let g2 = (y2 - y1).powi(2);
    (g1 + g2).sqrt()
}

/// Quit function run when the user clicks the quit button
#[tracing::instrument(skip(ctx, app))]
pub fn quit(ctx: &Context, app: &HappyChartState) -> Result<(), HappyChartError> {
    save_program_state(ctx, app)?;

    ctx.send_viewport_cmd(ViewportCommand::Close);
    Ok(())
}

#[tracing::instrument]
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

/// First load governs error states on its own, no need to read output
#[tracing::instrument(skip(app, ctx))]
pub fn first_load(app: &mut HappyChartState, ctx: &Context) {
    // all data we need to read one time on launch, all of this most of the time is unchanging throughout usage of the program, so it can only be recalculated on launch
    // for example, day quality averages do not need to change between launches
    app.first_load = false;

    match read_save_file() {
        Ok(save_file_days) => {
            app.days = save_file_days;
        }
        Err(err) => {
            app.error_states.push(err);
        }
    }

    app.days.sort_by(|day1, day2| {
        day1.get_date()
            .timestamp()
            .cmp(&day2.get_date().timestamp())
    });

    app.starting_length = app.days.len();
    let ls = read_last_session_save_file();
    app.open_modulus = ls.open_modulus;
    app.program_options = ls.program_options;
    app.last_open_date = ls.last_open_date;
    app.last_backup_date = ls.last_backup_date;
    if let Some(ver) = ls.last_version_checked {
        app.auto_update_seen_version = Some(ver);
    }
    app.tutorial_state = ls.tutorial_state;

    if Local::now()
        .signed_duration_since(ls.last_open_date)
        .num_hours()
        >= 12
    {
        match get_release_list() {
            Ok(list) => {
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
            Err(err) => {
                if !app.program_options.disable_update_list_error_showing {
                    app.error_states
                        .push(HappyChartError::UpdateReleaseList(err));
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
        match backup_program_state(ctx, app, false) {
            Ok(_) => {
                app.last_backup_date = Local::now();
            }
            Err(err) => {
                app.error_states.push(err);
            }
        }
    }

    app.remove_old_backup_files();

    app.stats.avg_weekdays.calc_averages(&app.days);
    app.stats
        .calc_streak(&app.days, app.program_options.streak_leniency);
}

#[tracing::instrument(skip_all)]
pub fn export_stats_to_csv(path: PathBuf, app: &mut HappyChartState) {
    match csv::WriterBuilder::new().from_path(&path) {
        Ok(mut export_writer) => {
            app.days.iter().for_each(|day_stat| {
                let written_data = &[
                    day_stat.get_date().to_string(),
                    day_stat.get_rating().to_string(),
                    day_stat.get_note().to_string(),
                    day_stat.get_mood_tags().iter().enumerate().fold(
                        String::new(),
                        |acc, (index, mood_tag)| {
                            if index == day_stat.get_mood_tags().len() - 1 {
                                format!("{}{}", acc, mood_tag.get_text())
                            } else {
                                format!("{}{},", acc, mood_tag.get_text())
                            }
                        },
                    ),
                ];

                // println!("{:?}", written_data);

                match export_writer.write_record(written_data) {
                    Ok(_) => {}
                    Err(err) => {
                        app.error_states
                            .push(HappyChartError::ExportIO(std::io::Error::from(err), None));
                    }
                }
            });

            if let Err(export_error) = export_writer.flush() {
                app.error_states
                    .push(HappyChartError::ExportIO(export_error, Some(path)));
            }
        }
        Err(export_error) => {
            app.error_states.push(HappyChartError::ExportIO(
                std::io::Error::from(export_error),
                Some(path),
            ));
        }
    }
}

#[tracing::instrument(skip(image))]
pub fn handle_screenshot_event(image: &Arc<ColorImage>) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Image", &["png", "jpeg", "jpg", "bmp", "tiff"])
        .save_file()
    {
        let _ = image::save_buffer(
            path,
            image.as_raw(),
            u32::try_from(image.width()).unwrap_or(u32::MAX),
            u32::try_from(image.height()).unwrap_or(u32::MAX),
            image::ColorType::Rgba8,
        );
    }
}

#[tracing::instrument(skip(ctx, app))]
pub fn backup_program_state(
    ctx: &Context,
    app: &HappyChartState,
    is_manual: bool,
) -> Result<(), HappyChartError> {
    let time = Local::now();
    save_program_state(ctx, app)?;
    let _ = fs::create_dir_all(&app.program_options.backup_save_path);
    let archive_file_name = get_backup_file_name(&time, is_manual);
    let file = File::create(
        app.program_options
            .backup_save_path
            .clone()
            .join(Path::new(&archive_file_name)),
    )
    .map_err(HappyChartError::SaveBackupIO)?;

    let mut arch = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
    if let Ok(mut old_save_file) = File::open(SAVE_FILE_NAME) {
        let _ = arch.start_file(SAVE_FILE_NAME, options);
        let mut old_file_bytes = vec![];
        let _ = old_save_file.read_to_end(&mut old_file_bytes);
        let _ = arch.write_all(&old_file_bytes);
    } else {
        // no old save file present, so we can just
    }
    let mut new_save_file =
        File::open(NEW_SAVE_FILE_NAME).map_err(HappyChartError::SaveBackupIO)?;
    let mut last_session_file =
        File::open(LAST_SESSION_FILE_NAME).map_err(HappyChartError::SaveBackupIO)?;
    let _ = arch.start_file(NEW_SAVE_FILE_NAME, options);
    let mut new_file_bytes = vec![];
    let _ = new_save_file.read_to_end(&mut new_file_bytes);
    let _ = arch.write_all(&new_file_bytes);
    let _ = arch.start_file(LAST_SESSION_FILE_NAME, options);
    let mut last_session_file_bytes = vec![];
    let _ = last_session_file.read_to_end(&mut last_session_file_bytes);
    let _ = arch.write_all(&last_session_file_bytes);
    let _ = arch.finish();

    Ok(())
}

#[tracing::instrument(skip(ctx, app))]
pub fn save_program_state(ctx: &Context, app: &HappyChartState) -> Result<(), HappyChartError> {
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
                .map(ToString::to_string)
        },
        last_backup_date: app.last_backup_date,
        tutorial_state: app.tutorial_state,
    };

    let session_ser =
        serde_json::to_string(&last_session).map_err(HappyChartError::Serialization)?;
    let last_session_path = Path::new(LAST_SESSION_FILE_NAME);

    let mut last_session_save_file = File::create(last_session_path).map_err(|io_error| {
        HappyChartError::WriteSaveFileIO(io_error, PathBuf::from(last_session_path))
    })?;

    last_session_save_file
        .write_all(session_ser.as_bytes())
        .map_err(|err| HappyChartError::WriteSaveFileIO(err, PathBuf::from(last_session_path)))?;

    let ser = serde_json::to_string(days).map_err(HappyChartError::Serialization)?;
    let save_path = Path::new(NEW_SAVE_FILE_NAME);

    let mut save_file = File::create(save_path)
        .map_err(|io_error| HappyChartError::WriteSaveFileIO(io_error, PathBuf::from(save_path)))?;

    save_file
        .write_all(ser.as_bytes())
        .map_err(|err| HappyChartError::WriteSaveFileIO(err, PathBuf::from(save_path)))?;

    Ok(())
}

#[tracing::instrument]
pub fn get_average_for_day_of_week(day_of_week: Weekday, days: &[ImprovedDayStat]) -> f32 {
    let ratings = days
        .iter()
        .filter(|stat| stat.get_date().weekday() == day_of_week)
        .map(|stat| stat.get_rating())
        .collect::<Vec<f32>>();

    ratings.iter().sum::<f32>() / ratings.len() as f32
}

#[tracing::instrument]
pub fn get_tutorial_highlight_glowing_color(offset: u8) -> Color32 {
    let now = SystemTime::now();

    let diff = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |diff| diff.as_secs());

    match (diff + u64::from(offset)) % 3 {
        0 => Color32::GRAY,
        1 => Color32::LIGHT_GRAY,
        _ => Color32::WHITE,
    }
}

#[tracing::instrument(skip_all)]
pub fn tutorial_button_colors(ui: &mut Ui) {
    ui.style_mut().visuals.widgets.inactive.bg_fill = get_tutorial_highlight_glowing_color(0);
    ui.style_mut().visuals.widgets.inactive.fg_stroke.color =
        get_tutorial_highlight_glowing_color(2);
    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = get_tutorial_lowlight_glowing_color(0);
}

#[tracing::instrument]
pub fn get_tutorial_lowlight_glowing_color(offset: u8) -> Color32 {
    let now = SystemTime::now();

    let diff = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |diff| diff.as_secs());

    match (diff + u64::from(offset)) % 3 {
        0 => Color32::from_gray(90),
        1 => Color32::from_gray(60),
        2 => Color32::from_gray(40),
        _ => Color32::DARK_GRAY,
    }
}

#[tracing::instrument]
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

#[tracing::instrument]
pub fn get_release_list() -> Result<Vec<Release>, Box<dyn Error>> {
    let list = self_update::backends::github::ReleaseList::configure()
        .repo_owner("CoryRobertson")
        .repo_name("happy_chart_rs")
        .build()?
        .fetch()?;
    #[cfg(debug_assertions)]
    println!("{:?}", list);
    Ok(list)
}

/// Reads the last session file, if exists, returns the deserialized contents, if it doesn't exist, returns a default `LastSession` struct.
#[tracing::instrument]
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
#[tracing::instrument]
pub fn read_save_file() -> Result<Vec<ImprovedDayStat>, HappyChartError> {
    let new_path = PathBuf::from(NEW_SAVE_FILE_NAME);
    let path = Path::new(SAVE_FILE_NAME);

    let mut file = match File::open(&new_path) {
        Ok(f) => f,
        Err(_) => match File::open(path) {
            Ok(f) => f,
            Err(_) => File::create(new_path.clone())
                .map_err(|io_error| HappyChartError::ReadSaveFileIO(io_error, new_path))?,
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
            return Ok(vec![]);
        }
    };

    // attempt to read old save file format
    match serde_json::from_str::<Vec<ImprovedDayStat>>(&s[0..read_len]) {
        Ok(vec) => {
            println!("found modern save file");
            // new save file format found, return it
            Ok(vec)
        }
        Err(err_improved) => {
            // not old save file format, attempt to read it as new save file format
            #[allow(deprecated)]
            match serde_json::from_str::<Vec<DayStat>>(&s[0..read_len]) {
                Ok(v) => {
                    println!("found legacy save file, converting");
                    // old save file format found, convert it into new save file format
                    Ok(v.into_iter()
                        .map(|old_day_stat| old_day_stat.into())
                        .collect::<Vec<ImprovedDayStat>>())
                }
                Err(err_old) => {
                    // cant read old or new save file format, so empty vec.
                    Err(HappyChartError::Deserialization(err_improved, err_old))
                }
            }
        }
    }
}

// thank you online example <3
#[tracing::instrument(skip(ui))]
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
