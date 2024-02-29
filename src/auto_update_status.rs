#[derive(Debug)]
pub enum AutoUpdateStatus {
    Checking,
    NotChecked,
    UpToDate(String),
    OutOfDate,
    Updated(String),
    Error(String),
}

impl Default for AutoUpdateStatus {
    fn default() -> Self {
        Self::NotChecked
    }
}

impl AutoUpdateStatus {
    pub fn to_text(&self) -> String {
        match self {
            Self::Checking => "Checking".to_string(),
            Self::NotChecked => "Not Checked".to_string(),
            Self::UpToDate(ver) => {
                format!("Up to date: {}", ver)
            }
            Self::Updated(ver) => {
                format!("Updated: {}", ver)
            }
            Self::Error(err) => {
                format!("Error: {}", err)
            }
            Self::OutOfDate => "Out of date".to_string(),
        }
    }
}
