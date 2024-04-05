use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UserPromptStates {
    pub tried_logging: bool,
    pub tried_backups: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for UserPromptStates {
    fn default() -> Self {
        Self {
            tried_logging: false,
            tried_backups: false,
        }
    }
}
