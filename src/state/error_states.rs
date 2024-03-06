use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug)]
pub enum HappyChartError {
    Serialization(serde_json::Error),
    Deserialization(serde_json::Error, Option<serde_json::Error>),
    ReadSaveFileIO(std::io::Error, PathBuf),
    WriteSaveFileIO(std::io::Error, PathBuf),
    UpdateReleaseList(Box<dyn Error>),
    SaveBackupIO(std::io::Error),
    ExportIO(std::io::Error, Option<PathBuf>),
    /// Error thrown if the save file read was unreadable, suggesting it is encrypted, this error does not open the regular error screen, and instead prompts the user to enter an encryption key
    EncryptedSaveFile(Vec<u8>),
    EncryptionKeysDontMatch,
    EncryptionError(cocoon::Error),
    DecryptionError(cocoon::Error),
    EncryptKeyTooShort {
        /// true if the primary key is too short
        primary_key_problem: bool,
        /// true if the secondary key is too short
        secondary_key_problem: bool,
    },
    EncryptKeyTooLong {
        /// true if the primary key is too long
        primary_key_problem: bool,
        /// true if the secondary key is too long
        secondary_key_problem: bool,
    },
}

impl Display for HappyChartError {
    #[tracing::instrument(skip(self, f))]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Serialization(err) => {
                    format!("HappyChartError::SerializationError {}", err)
                }
                Self::ReadSaveFileIO(err, path) => {
                    format!(
                        "HappyChartError::ReadSaveFileIOError {} {}",
                        err,
                        path.to_str().unwrap_or("UNABLE TO DISPLAY PATH")
                    )
                }
                Self::WriteSaveFileIO(err, path) => {
                    format!(
                        "HappyChartError::WriteSaveFileIOError {} {}",
                        err,
                        path.to_str().unwrap_or("UNABLE TO DISPLAY PATH")
                    )
                }
                Self::UpdateReleaseList(err) => {
                    format!("HappyChartError::UpdateReleaseListError {}", err)
                }
                Self::SaveBackupIO(err) => {
                    format!("HappyChartError::SaveBackupIOError {}", err)
                }
                Self::Deserialization(improved_save_error, old_save_error) => {
                    format!(
                        "HappyChartError::DeserializationError {} {:?}",
                        improved_save_error, old_save_error
                    )
                }
                Self::ExportIO(err, path) => {
                    format!(
                        "HappyChartError::ExportIO {} {}",
                        err,
                        path.as_ref().map_or("UNABLE TO DISPLAY PATH", |p| p
                            .to_str()
                            .unwrap_or("UNABLE TO DISPLAY PATH"))
                    )
                }
                Self::EncryptedSaveFile(_) => {
                    "HappyChartError::EncryptedSaveFile".to_string()
                }
                Self::EncryptionKeysDontMatch => {
                    "HappyChartError::EncryptionKeysDontMatch".to_string()
                }
                Self::EncryptionError(err) => {
                    format!("HappyChartError::EncryptionError {:?}", err)
                }
                Self::DecryptionError(err) => {
                    format!("HappyChartError::DecryptionError {:?}", err)
                }
                Self::EncryptKeyTooShort { .. } => {
                    "HappyChartError::EncryptKeyTooShort".to_string()
                }
                Self::EncryptKeyTooLong { .. } => {
                    "HappyChartError::EncryptKeyTooLong".to_string()
                }
            }
        )
    }
}
