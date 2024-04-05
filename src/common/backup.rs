use crate::common::save::save_program_state;
use crate::prelude::HappyChartState;
use crate::state::error_states::HappyChartError;
use crate::{
    BACKUP_FILENAME_PREFIX, BACKUP_FILE_EXTENSION, LAST_SESSION_FILE_NAME, MANUAL_BACKUP_SUFFIX,
    NEW_SAVE_FILE_NAME, SAVE_FILE_NAME,
};
use chrono::{DateTime, Datelike, Local};
use egui::Context;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tracing::info;
use zip::write::FileOptions;
use zip::CompressionMethod;

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
    let archive_path = app
        .program_options
        .backup_save_path
        .clone()
        .join(Path::new(&archive_file_name));
    let file = File::create(&archive_path).map_err(HappyChartError::SaveBackupIO)?;

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

    info!("Successfully saved backup in path {:?}", archive_path);

    Ok(())
}
