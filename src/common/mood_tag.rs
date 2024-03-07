use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(
    Debug, Clone, Copy, Hash, Serialize, Deserialize, Ord, PartialOrd, PartialEq, Eq, EnumIter,
)]
#[non_exhaustive]
pub enum MoodTag {
    // Happy category
    Happy,
    Caring,
    Grateful,
    Excited,
    // Sad category
    Sad,
    Lonely,
    Hurt,
    Disappointed,
    // Loved category
    Loved,
    Respected,
    Valued,
    Accepted,
    // Confident category
    Confident,
    Brave,
    Hopeful,
    Powerful,
    // Playful category
    Playful,
    Creative,
    Curious,
    Affectionate,
    // Embarrassed category
    Embarrassed,
    Ashamed,
    Excluded,
    Guilty,
    // Angry category
    Angry,
    Bored,
    Jealous,
    Annoyed,
    // Scared category
    Scared,
    Anxious,
    Powerless,
    Overwhelmed,
}

impl MoodTag {
    #[tracing::instrument]
    pub fn get_mood_by_name(text: &str) -> Option<Self> {
        let search_term = text.to_lowercase();
        Self::iter().find(|mood| format!("{:?}", mood).to_lowercase().contains(&search_term))
    }

    #[tracing::instrument]
    pub fn get_text(&self) -> String {
        format!("{:?}", self)
    }

    #[tracing::instrument]
    pub fn get_emoji_text(&self) -> &str {
        match self {
            Self::Happy => "😃",
            Self::Caring => "💓",
            Self::Grateful => "🙇",
            Self::Excited => "😆",
            Self::Sad => "😢",
            Self::Lonely => "🚫🍺",
            Self::Hurt => "😧",
            Self::Disappointed => "😞",
            Self::Loved => "😍",
            Self::Respected => "🚩",
            Self::Valued => "💲🙂",
            Self::Accepted => "👍",
            Self::Confident => "😎",
            Self::Brave => "🦸",
            Self::Hopeful => "✌️",
            Self::Powerful => "💪",
            Self::Playful => "⛹️",
            Self::Creative => "🎨",
            Self::Curious => "🐈",
            Self::Affectionate => "💌",
            Self::Embarrassed => "😳",
            Self::Ashamed => "😞",
            Self::Excluded => "🙅",
            Self::Guilty => "😰",
            Self::Angry => "😠",
            Self::Bored => "😐",
            Self::Jealous => "😒",
            Self::Annoyed => "🙄",
            Self::Scared => "😰",
            Self::Anxious => "😓",
            Self::Powerless => "🚫⚡️",
            Self::Overwhelmed => "😬",
        }
    }
}
