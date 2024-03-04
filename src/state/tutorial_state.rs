use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Ord, PartialOrd, PartialEq, Eq)]
pub enum TutorialGoal {
    BeginTutorial,
    AddRating(bool),
    OpenSelectMood,
    SelectAMood,
    WriteNote,
    AddDay,
    OpenOptions,
    DoneWithTutorial,
    TutorialClosed,
}

impl Default for TutorialGoal {
    #[tracing::instrument]
    fn default() -> Self {
        Self::BeginTutorial
    }
}

impl TutorialGoal {}
