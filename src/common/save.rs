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
use tracing::{error, info, warn};

#[tracing::instrument(skip(ctx, app))]
pub fn save_program_state(ctx: &Context, app: &HappyChartState) -> Result<(), HappyChartError> {
    info!("Saving program state...");
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

    info!("Last session save file written to: {:?}", last_session_path);

    // only check for save file encryption issues if the user has encryption enabled, and we have already written the last session file
    if app.program_options.encrypt_save_file {
        encryption_save_file_checks(app)?;
    }

    let ser = serde_json::to_string(days).map_err(HappyChartError::Serialization)?;
    let save_path = Path::new(NEW_SAVE_FILE_NAME);

    let mut save_file = File::create(save_path)
        .map_err(|io_error| HappyChartError::WriteSaveFileIO(io_error, PathBuf::from(save_path)))?;

    info!("Creating save file at path: {:?}", save_path);

    if app.program_options.encrypt_save_file {
        info!("Save file encryption enabled, encrypting...");
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
        info!("Save file encryption disabled, saving...");
        save_file
            .write_all(ser.as_bytes())
            .map_err(|err| HappyChartError::WriteSaveFileIO(err, PathBuf::from(save_path)))?;
    }

    Ok(())
}

/// Reads the last session file, if exists, returns the deserialized contents, if it doesn't exist, returns a default `LastSession` struct.
#[tracing::instrument]
pub fn read_last_session_save_file() -> LastSession {
    info!("Reading last session save file");
    let path = Path::new(LAST_SESSION_FILE_NAME);

    let mut file = match File::open(path) {
        // try to open save file
        Ok(f) => {
            info!(
                "Last session save file found and opened successfully at path: {:?}",
                path
            );
            f
        }
        Err(_) => {
            match File::create(path) {
                // save file wasn't found, make one
                Ok(f) => {
                    info!(
                        "Last session save file not found, creating one at path: {:?}",
                        path
                    );
                    f
                }
                Err(_) => {
                    error!(
                        "Error creating save file at path: {:?}, using a default session save file",
                        path
                    );
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
            info!("Read last session save file successfully");
        }
        Err(_) => {
            error!("Failed to read file for last session, using a default session");
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

    info!(
        "Reading save file at path: {:?} with a fallback path of {:?}",
        new_path, path
    );

    let mut file = match File::open(&new_path) {
        Ok(f) => {
            info!("Successfully opening save file at new path");
            f
        }
        Err(e) => {
            match File::open(path) {
                Ok(f) => {
                    info!("Save file not found at new path, instead found at fallback path successfully");
                    f
                }
                Err(e1) => {
                    error!("Error finding save file in {:?} or fallback path of {:?}, errors in reading: {:?} and {:?}", new_path, path,e,e1);
                    File::create(new_path.clone())
                        .map_err(|io_error| HappyChartError::ReadSaveFileIO(io_error, new_path))?
                }
            }
        }
    };

    let mut s = vec![];
    let read_len = match file.read_to_end(&mut s) {
        Ok(read_len) => {
            info!("Successfully read save file of size: {}", read_len);
            read_len
        }
        Err(e) => {
            error!("Unable to read save file: {:?}", e);
            return Ok(vec![]);
        }
    };

    // attempt to read old save file format
    match serde_json::from_slice::<Vec<ImprovedDayStat>>(&s[0..read_len]) {
        Ok(vec) => {
            info!("Found modern save file");
            // new save file format found, return it
            Ok(vec)
        }
        Err(_err_improved) => {
            // not old save file format, attempt to read it as new save file format
            #[allow(deprecated)]
            match serde_json::from_slice::<Vec<DayStat>>(&s[0..read_len]) {
                Ok(v) => {
                    info!("Found legacy save file, converting now");
                    // old save file format found, convert it into new save file format
                    Ok(v.into_iter()
                        .map(|old_day_stat| old_day_stat.into())
                        .collect::<Vec<ImprovedDayStat>>())
                }
                Err(e) => {
                    warn!(
                        "Error deserializing save file, most likely it is encrypted? {:?}",
                        e
                    );
                    // cant read old or new save file format, so empty vec.

                    return Err(HappyChartError::EncryptedSaveFile(
                        (s[0..read_len]).to_vec(),
                    ));
                }
            }
        }
    }
}
