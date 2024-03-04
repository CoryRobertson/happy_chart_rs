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
        MoodTag::iter().find(|mood| format!("{:?}", mood).to_lowercase().contains(&search_term))
    }

    #[tracing::instrument]
    pub fn get_text(&self) -> String {
        format!("{:?}", self)
    }

    #[tracing::instrument]
    pub fn get_emoji_text(&self) -> &str {
        match self {
            MoodTag::Happy => "😃",
            MoodTag::Caring => "💓",
            MoodTag::Grateful => "🙇",
            MoodTag::Excited => "😆",
            MoodTag::Sad => "😢",
            MoodTag::Lonely => "🚫🍺",
            MoodTag::Hurt => "😧",
            MoodTag::Disappointed => "😞",
            MoodTag::Loved => "😍",
            MoodTag::Respected => "🚩",
            MoodTag::Valued => "💲🙂",
            MoodTag::Accepted => "👍",
            MoodTag::Confident => "😎",
            MoodTag::Brave => "🦸",
            MoodTag::Hopeful => "✌️",
            MoodTag::Powerful => "💪",
            MoodTag::Playful => "⛹️",
            MoodTag::Creative => "🎨",
            MoodTag::Curious => "🐈",
            MoodTag::Affectionate => "💌",
            MoodTag::Embarrassed => "😳",
            MoodTag::Ashamed => "😞",
            MoodTag::Excluded => "🙅",
            MoodTag::Guilty => "😰",
            MoodTag::Angry => "😠",
            MoodTag::Bored => "😐",
            MoodTag::Jealous => "😒",
            MoodTag::Annoyed => "🙄",
            MoodTag::Scared => "😰",
            MoodTag::Anxious => "😓",
            MoodTag::Powerless => "🚫⚡️",
            MoodTag::Overwhelmed => "😬",
        }
    }
}
