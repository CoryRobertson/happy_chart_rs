pub enum AutoUpdateStatus {
    Checking,
    NotChecked,
    UpToDate(String),
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
            AutoUpdateStatus::Checking => "Checking".to_string(),
            AutoUpdateStatus::NotChecked => "NotChecked".to_string(),
            AutoUpdateStatus::UpToDate(ver) => {
                format!("Up to date: {}", ver)
            }
            AutoUpdateStatus::Updated(ver) => {
                format!("Updated: {}", ver)
            }
            AutoUpdateStatus::Error(err) => {
                format!("Error: {}", err)
            }
        }
    }
}
