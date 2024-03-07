use crate::common::encryption::encryption_save_file_checks;
use crate::common::last_session::LastSession;
#[allow(deprecated)]
use crate::day_stats::daystat::DayStat;
use crate::prelude::{HappyChartState, ImprovedDayStat};
use crate::state::error_states::HappyChartError;
use crate::{LAST_SESSION_FILE_NAME, NEW_SAVE_FILE_NAME, SAVE_FILE_NAME};
use chrono::Local;
use cocoon::MiniCocoon;
use eframe::emath::{Pos2, Rect};
use egui::Context;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

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

    // only check for save file encryption issues if the user has encryption enabled, and we have already written the last session file
    if app.program_options.encrypt_save_file {
        encryption_save_file_checks(app)?;
    }

    let ser = serde_json::to_string(days).map_err(HappyChartError::Serialization)?;
    let save_path = Path::new(NEW_SAVE_FILE_NAME);

    let mut save_file = File::create(save_path)
        .map_err(|io_error| HappyChartError::WriteSaveFileIO(io_error, PathBuf::from(save_path)))?;

    if app.program_options.encrypt_save_file {
        let mut key = app.encryption_key.to_string();
        if key.len() < 32 {
            key.push_str("00000000000000000000000000000000");
        }

        let mut cocoon = MiniCocoon::from_key(&key.as_bytes()[0..32], &[0; 32]);
        let decrypt = cocoon
            .wrap(ser.as_ref())
            .map_err(HappyChartError::EncryptionError)?;
        save_file
            .write_all(&decrypt)
            .map_err(|err| HappyChartError::WriteSaveFileIO(err, PathBuf::from(save_path)))?;
    } else {
        save_file
            .write_all(ser.as_bytes())
            .map_err(|err| HappyChartError::WriteSaveFileIO(err, PathBuf::from(save_path)))?;
    }

    Ok(())
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

    let mut s = vec![];
    let read_len = match file.read_to_end(&mut s) {
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
    match serde_json::from_slice::<Vec<ImprovedDayStat>>(&s[0..read_len]) {
        Ok(vec) => {
            println!("found modern save file");
            // new save file format found, return it
            Ok(vec)
        }
        Err(_err_improved) => {
            // not old save file format, attempt to read it as new save file format
            #[allow(deprecated)]
            match serde_json::from_slice::<Vec<DayStat>>(&s[0..read_len]) {
                Ok(v) => {
                    println!("found legacy save file, converting");
                    // old save file format found, convert it into new save file format
                    Ok(v.into_iter()
                        .map(|old_day_stat| old_day_stat.into())
                        .collect::<Vec<ImprovedDayStat>>())
                }
                Err(_err_old) => {
                    // cant read old or new save file format, so empty vec.

                    return Err(HappyChartError::EncryptedSaveFile(
                        (s[0..read_len]).to_vec(),
                    ));
                }
            }
        }
    }
}
