use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::fmt::{Debug, Formatter};
use std::thread::JoinHandle;
use strum_macros::EnumIter;

#[derive(Default)]
pub struct FeedbackUiState {
    pub happy_chart_feedback: HappyChartFeedback,
    pub showing_feedback_screen: bool,
    pub submit_thread: Cell<Option<JoinHandle<()>>>,
}

impl Debug for FeedbackUiState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let _ = write!(
            f,
            "{:?},{:?}",
            self.happy_chart_feedback, self.showing_feedback_screen
        );

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HappyChartFeedback {
    message: Message,
    rating: FeedbackRating,
}

impl HappyChartFeedback {
    pub fn get_message_mut(&mut self) -> &mut String {
        &mut self.message.0
    }

    pub fn get_feedback(&self) -> FeedbackRating {
        self.rating
    }

    pub fn set_feedback(&mut self, new_rating: FeedbackRating) {
        self.rating = new_rating;
    }
}

impl Default for HappyChartFeedback {
    fn default() -> Self {
        Self {
            message: Message("".to_string()),
            rating: FeedbackRating::VeryPositive,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message(pub String);

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, EnumIter, Ord, PartialOrd, Eq, PartialEq, Hash,
)]
pub enum FeedbackRating {
    VeryPositive,
    SomewhatPositive,
    Neutral,
    SomewhatNegative,
    VeryNegative,
}
