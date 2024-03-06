use crate::day_stats::improved_daystat::ImprovedDayStat;
use crate::prelude::HappyChartState;
use crate::state::error_states::HappyChartError;
use cocoon::MiniCocoon;
use egui::{TextEdit, Ui};

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
        ui.add(TextEdit::singleline(&mut app.encryption_key).password(true));

        if ui.button("Unlock").clicked() {
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
