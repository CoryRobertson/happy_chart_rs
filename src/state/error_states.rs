use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug)]
pub enum HappyChartError {
    Serialization(serde_json::Error),
    Deserialization(serde_json::Error, serde_json::Error),
    ReadSaveFileIO(std::io::Error, PathBuf),
    WriteSaveFileIO(std::io::Error, PathBuf),
    UpdateReleaseList(Box<dyn Error>),
    SaveBackupIO(std::io::Error),
}

impl Display for HappyChartError {
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
                        "HappyChartError::DeserializationError {} {}",
                        improved_save_error, old_save_error
                    )
                }
            }
        )
    }
}
