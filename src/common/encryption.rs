use crate::prelude::{HappyChartState, ImprovedDayStat};
use crate::state::error_states::HappyChartError;
use crate::{MAX_ENCRYPT_KEY_LENGTH, MIN_ENCRYPT_KEY_LENGTH};
use cocoon::MiniCocoon;
use tracing::info;

#[tracing::instrument(skip_all)]
pub fn decrypt_save_file(
    app: &HappyChartState,
    encrypted_data: &[u8],
) -> Result<Vec<ImprovedDayStat>, HappyChartError> {
    info!("Decrypting save file");
    let mut key = app.encryption_key.to_string();
    if key.len() < 32 {
        key.push_str("00000000000000000000000000000000");
    }

    let cocoon = MiniCocoon::from_key(&key.as_bytes()[0..32], &[0; 32]);
    let unwrapped = cocoon
        .unwrap(encrypted_data)
        .map_err(HappyChartError::DecryptionError)?;

    info!("Successfully decrypted save file, deserializing now.");

    serde_json::from_slice::<Vec<ImprovedDayStat>>(&unwrapped)
        .map_err(|err| HappyChartError::Deserialization(err, None))
}

#[tracing::instrument(skip_all)]
pub fn encryption_save_file_checks(app: &HappyChartState) -> Result<(), HappyChartError> {
    // keys are not the same
    if app.encryption_key.ne(&app.encryption_key_second_check) {
        return Err(HappyChartError::EncryptionKeysDontMatch);
    }

    // either key is too short
    if app.encryption_key.len() < MIN_ENCRYPT_KEY_LENGTH
        && app.encryption_key_second_check.len() < MIN_ENCRYPT_KEY_LENGTH
    {
        return Err(HappyChartError::EncryptKeyTooShort {
            primary_key_problem: app.encryption_key.len() < MIN_ENCRYPT_KEY_LENGTH,
            secondary_key_problem: app.encryption_key_second_check.len() < MIN_ENCRYPT_KEY_LENGTH,
        });
    }

    // either key is too long
    if app.encryption_key.len() > MAX_ENCRYPT_KEY_LENGTH
        && app.encryption_key_second_check.len() > MAX_ENCRYPT_KEY_LENGTH
    {
        return Err(HappyChartError::EncryptKeyTooLong {
            primary_key_problem: app.encryption_key.len() > MAX_ENCRYPT_KEY_LENGTH,
            secondary_key_problem: app.encryption_key_second_check.len() > MAX_ENCRYPT_KEY_LENGTH,
        });
    }

    Ok(())
}
