use crate::common::{encryption_save_file_checks, tutorial_button_colors};
use crate::day_stats::improved_daystat::ImprovedDayStat;
use crate::prelude::HappyChartState;
use crate::state::error_states::HappyChartError;
use crate::{MAX_ENCRYPT_KEY_LENGTH, MIN_ENCRYPT_KEY_LENGTH};
use cocoon::MiniCocoon;
use eframe::epaint::Color32;
use egui::{RichText, TextEdit, Ui};

pub fn draw_decryption_screen(
    ui: &mut Ui,
    app: &mut HappyChartState,
) -> Result<(), HappyChartError> {
    let mut save_file_decrypted_successfully: Option<usize> = None;
    if let Some((index, HappyChartError::EncryptedSaveFile(encrypted_data))) = app
        .error_states
        .iter()
        .enumerate()
        .find(|(_, err)| matches!(err, HappyChartError::EncryptedSaveFile(_)))
    {
        ui.label("Encryption key: ");
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
            app.encryption_key_second_check = app.encryption_key.to_string();
            let mut key = app.encryption_key.to_string();
            if key.len() < 32 {
                key.push_str("00000000000000000000000000000000");
            }

            let cocoon = MiniCocoon::from_key(&key.as_bytes()[0..32], &[0; 32]);
            let unwrapped = cocoon
                .unwrap(encrypted_data)
                .map_err(HappyChartError::DecryptionError)?;
            app.days = serde_json::from_slice::<Vec<ImprovedDayStat>>(&unwrapped)
                .map_err(|err| HappyChartError::Deserialization(err, None))?;
            save_file_decrypted_successfully = Some(index);
        }
    }

    if let Some(_index) = save_file_decrypted_successfully {
        app.error_states.retain(|err| {
            !matches!(err, HappyChartError::DecryptionError(_))
                && !matches!(err, HappyChartError::EncryptedSaveFile(_))
        });
        app.stats
            .calc_streak(&app.days, app.program_options.streak_leniency);
        app.stats.avg_weekdays.calc_averages(&app.days);
    }

    Ok(())
}

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
