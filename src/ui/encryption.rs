use crate::common::color::tutorial_button_colors;
use crate::common::encryption::{decrypt_save_file, encryption_save_file_checks};
use crate::common::first_load;
use crate::prelude::HappyChartState;
use crate::state::error_states::HappyChartError;
use crate::{MAX_ENCRYPT_KEY_LENGTH, MIN_ENCRYPT_KEY_LENGTH};
use eframe::epaint::Color32;
use egui::{Context, RichText, TextEdit, Ui};
use tracing::info;

#[tracing::instrument(skip_all)]
pub fn draw_decryption_screen(
    ui: &mut Ui,
    app: &mut HappyChartState,
    ctx: &Context,
) -> Result<(), HappyChartError> {
    let mut save_file_decrypted_successfully: Option<usize> = None;
    if let Some((index, HappyChartError::EncryptedSaveFile(encrypted_data))) = app
        .error_states
        .iter()
        .enumerate()
        .find(|(_, err)| matches!(err, HappyChartError::EncryptedSaveFile(_)))
    {
        ui.label("Encryption key:");
        let key_input_resp = ui.add(TextEdit::singleline(&mut app.encryption_key).password(true));
        let unlock_button = ui.button("Unlock");

        // force the user to focus on the password input if they have not put in a key
        if app.encryption_key.is_empty() {
            key_input_resp.request_focus();
        }

        // the user can either click the unlock button, or lose focus on the key text edit object while having a key
        if unlock_button.clicked()
            || (!app.encryption_key.is_empty() && key_input_resp.lost_focus())
        {
            info!("Unlock button clicked");
            let decrypted_save = decrypt_save_file(app, encrypted_data)?;

            // set the second key equal to the first key so after the user unlocks the save file, they don't have to re-type their password
            app.encryption_key_second_check = app.encryption_key.to_string();

            app.days = decrypted_save;

            save_file_decrypted_successfully = Some(index);
        }
    }

    if let Some(_index) = save_file_decrypted_successfully {
        app.error_states.retain(|err| {
            !matches!(err, HappyChartError::DecryptionError(_))
                && !matches!(err, HappyChartError::EncryptedSaveFile(_))
        });
        first_load(app, ctx, false);
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn draw_fix_encryption_keys_screen(ui: &mut Ui, app: &mut HappyChartState) {
    let error_issue = encryption_save_file_checks(app);

    ui.horizontal(|ui| {
        ui.label("Encryption key:");
        if let Err(HappyChartError::EncryptKeyTooShort {
            primary_key_problem: true,
            ..
        }) = error_issue
        {
            tutorial_button_colors(ui);
        }
        if let Err(HappyChartError::EncryptKeyTooLong {
            primary_key_problem: true,
            ..
        }) = error_issue
        {
            tutorial_button_colors(ui);
        }
        ui.add(TextEdit::singleline(&mut app.encryption_key).password(true));
    });
    ui.horizontal(|ui| {
        ui.label("Encryption key a second time:");
        if let Err(HappyChartError::EncryptKeyTooShort {
            secondary_key_problem: true,
            ..
        }) = error_issue
        {
            tutorial_button_colors(ui);
        }
        if let Err(HappyChartError::EncryptKeyTooLong {
            secondary_key_problem: true,
            ..
        }) = error_issue
        {
            tutorial_button_colors(ui);
        }
        ui.add(TextEdit::singleline(&mut app.encryption_key_second_check).password(true));
    });

    if app.encryption_key_second_check.len() < MIN_ENCRYPT_KEY_LENGTH
        && app.encryption_key.len() < MIN_ENCRYPT_KEY_LENGTH
    {
        ui.label(
            RichText::new(format!(
                "Encryption keys are too short, minimum length of {} characters",
                MIN_ENCRYPT_KEY_LENGTH
            ))
            .color(Color32::LIGHT_RED),
        );
    }
    if app.encryption_key_second_check.len() > MAX_ENCRYPT_KEY_LENGTH
        && app.encryption_key.len() > MAX_ENCRYPT_KEY_LENGTH
    {
        ui.label(
            RichText::new(format!(
                "Encryption keys are too long, maximum length of {} characters",
                MIN_ENCRYPT_KEY_LENGTH
            ))
            .color(Color32::LIGHT_RED),
        );
    }
    if app.encryption_key != app.encryption_key_second_check {
        ui.label(RichText::new("Encryption keys do not match").color(Color32::LIGHT_RED));
    }
}
