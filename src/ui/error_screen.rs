use crate::options::program_options::ProgramOptions;
use crate::state::error_states::HappyChartError;
use crate::state::happy_chart_state::HappyChartState;
use crate::ui::encryption::draw_fix_encryption_keys_screen;
use crate::{MAX_ENCRYPT_KEY_LENGTH, MIN_ENCRYPT_KEY_LENGTH};
use egui::Ui;

#[tracing::instrument(skip(ui, app))]
pub fn draw_error_screen(app: &mut HappyChartState, ui: &mut Ui) {
    app.error_states.iter()
        .for_each(|error_state| {
        ui.label(format!("{}", error_state));
        ui.separator();
        match error_state {
            HappyChartError::Serialization(_) => {
                ui.label("Happy chart encountered an error while attempting to serialize the current save file.");
                ui.label("This error should be reported on github, found linked in the about page. Please make an issue and report what type of chart entry caused this.");
                ui.label("A possible reason this error could occur would be if some user input in the program contains an invalid character, or invalid data somehow.");
            }
            HappyChartError::Deserialization(improved_save_error, old_save_error) => {
                ui.label("Happy chart encountered an error while reading a save file, the save file could be corrupted, or contains invalid data somehow.");
                ui.label("Restoring a backup could be a valid solution, or manually editing the save file to check for validity, though this is not recommended.");
                ui.label(format!("The most likely save file error is\n {}", improved_save_error));
                ui.label(format!("However, if the last version you used of the program was a very old version, it could instead be this error:\n {:?}", old_save_error));

            }
            HappyChartError::ReadSaveFileIO(_, path) => {
                ui.horizontal(|ui| {
                    ui.label("Happy chart was unable to successfully read from this path: ");
                    ui.label(path.to_str().unwrap_or("UNABLE TO DISPLAY PATH"));
                });
                ui.label("This error most likely occurs when Happy chart tries to read from an invalid path,\
                             or when the program lacks requires permissions to read the save file.\
                              One solution is to move the program and its save files such that it is in an environment where it can read the file.\
                              Another would be to give the program needed permissions to read the file");
                ui.label("Happy chart will most likely not function properly at all while this error is present.\
                             This error occurs during the reading process of the chart data. If the save file is in the same location as the path above,\
                             then its possible the save file has specific permissions which restrict Happy chart from reading it.\
                              If somehow the save file is corrupted in such a way where it cant be read, then check your backups folder if one is present.");
            }
            HappyChartError::WriteSaveFileIO(_, path) => {
                ui.horizontal(|ui| {
                    ui.label("Happy chart was unable to successfully write to this path: ");
                    ui.label(path.to_str().unwrap_or("UNABLE TO DISPLAY PATH"));
                });
                ui.label("This error most likely occurs when Happy chart does not have permissions to save to the given save location.\
                             One solution is to move the program to a folder where it does have permissions to do so, \
                             if that is not an option, you can give the program permissions to write to this folder.");
                ui.label("While this error is present, it is unlikely that changes to happy chart will successfully save,\
                             so this is a rather serious error if changes were made to your chart.");

            }
            HappyChartError::UpdateReleaseList(_) => {
                ui.label("Happy chart was unable to get a release list from github, this could be a github error, a lack of internet connection, or something similar.");
                ui.label("While this error is present, it is unlikely that automatic updates will work, other than that, the program should work just fine. :)");
                ui.label("This should be safe, its more so just so the user knows they are not getting automatic updates");
                ui.label("If you use the program regularly without an active internet connection, this would be a good idea to ignore. :)");
                if ui.button("Ignore this error from now on").on_hover_text("The program will no longer report this error. After clicking this, when ever this error next occurs, it will be ignored.").clicked() {
                    app.program_options.disable_update_list_error_showing = true;
                }
            }
            HappyChartError::SaveBackupIO(_) => {
                ui.label("An error occurred while attempting to save a backup shown above.");
                ui.horizontal(|ui| {
                    ui.label("Backup path: ");
                    ui.label(app.program_options.backup_save_path.to_str().unwrap_or_default());
                });

                if ui.button("Set new backup path").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_directory("./")
                        .set_title("Set the location where a backup will be stored")
                        .pick_folder() {
                        app.program_options.backup_save_path = path;
                    }
                }

                if ui.button("Reset backup path to default").on_hover_text("The default path is a local folder called \'backups\'").clicked() {
                    app.program_options.backup_save_path = ProgramOptions::default().backup_save_path;
                }

                if app.program_options.backup_save_path
                    .try_exists()
                    .is_ok_and(|exists| exists) {
                    ui.label("Happy chart believes that this path is valid, so its possible that happy chart does not have permission to write and read to that folder.");
                } else {
                    ui.label("Happy chart does not believe that the backup path exists, it might be missing permissions, or the path could be invalid.");
                }
            }
            HappyChartError::ExportIO(export_io_error, path) => {
                ui.label("An error occurred while attempting to export your save data to another format");
                match path {
                    None => {}
                    Some(p) => {
                        ui.horizontal(|ui| {
                            ui.label("Happy chart was unable to successfully write to this path: ");
                            ui.label(p.to_str().unwrap_or("UNABLE TO DISPLAY PATH"));
                        });
                    }
                }

                ui.label(&format!("The full IO error is: {}", export_io_error));
            }
            HappyChartError::EncryptedSaveFile(_) => {
                ui.label("Your save file is encrypted.");
            }
            HappyChartError::EncryptionKeysDontMatch => {
                ui.label("Your encryption keys do not match, please check that they do. Or disable save file encryption.");
            }
            HappyChartError::EncryptionError(err) => {
                ui.label(format!("An error occurred while encrypting your save file: {:?}", err));
            }
            HappyChartError::DecryptionError(err) => {
                ui.label(format!("An error occurred while decrypting your save file: {:?}", err));
            }
            HappyChartError::EncryptKeyTooShort{ .. } => {
                ui.label(format!("Your encryption key is too short, you can either disable save file encryption, or add to its length. The minimum length is {}.", MIN_ENCRYPT_KEY_LENGTH));
            }
            HappyChartError::EncryptKeyTooLong{ .. } => {
                ui.label(format!("Your encryption key is too long, you can either disable save file encryption, or remove from its length. The maximum length is {}.", MAX_ENCRYPT_KEY_LENGTH));
            }
        }
        ui.separator();
    });

    if app.error_states.iter().any(|err| {
        matches!(
            err,
            HappyChartError::EncryptKeyTooShort { .. }
                | HappyChartError::EncryptionKeysDontMatch
                | HappyChartError::EncryptKeyTooLong { .. }
        )
    }) {
        draw_fix_encryption_keys_screen(ui, app);
        ui.checkbox(
            &mut app.program_options.encrypt_save_file,
            "Encrypt save file",
        );
    }

    if ui
        .button("Close and dismiss errors")
        .on_hover_text("This screen will pop up again if a new error occurs")
        .clicked()
    {
        app.error_states.clear();
    }
}
