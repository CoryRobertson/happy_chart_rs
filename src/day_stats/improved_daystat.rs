#[allow(deprecated)]
use crate::day_stats::daystat::DayStat;
use crate::prelude::MoodTag;
use crate::state::activities::Activity;
use chrono::{DateTime, Datelike, Local, Timelike};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct ImprovedDayStat {
    rating: f32,
    date: DateTime<Local>,
    note: String,
    mood_tags: Vec<MoodTag>,
    activities: Vec<Activity>,
}

impl Default for ImprovedDayStat {
    fn default() -> Self {
        Self {
            rating: 0.0,
            date: Local::now(),
            note: "DEFAULT NOTE".to_string(),
            mood_tags: vec![],
            activities: vec![],
        }
    }
}

impl Display for ImprovedDayStat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Date: ")?;
        f.write_str(&format!(
            "{}-{}-{} {}:{:02} {}",
            self.date.month(),
            self.date.day(),
            self.date.year(),
            self.date.hour12().1,
            self.date.minute(),
            {
                if self.date.hour12().0 {
                    "PM"
                } else {
                    "AM"
                }
            }
        ))?;
        f.write_str("\n")?;
        f.write_str("Rating: ")?;
        f.write_str(&self.rating.to_string())?;
        f.write_str("\n")?;
        if !self.note.is_empty() {
            f.write_str(&self.note)?;
            f.write_str("\n")?;
        }
        if !self.mood_tags.is_empty() {
            f.write_str("Mood tags:\n")?;
            for mood in &self.mood_tags {
                f.write_str(&format!("\t{:?}\n", mood))?;
            }
        }
        if !self.activities.is_empty() {
            f.write_str("Activities:\n")?;
            for act in &self.activities {
                f.write_str(&format!("\t{}\n", act))?;
            }
        }
        Ok(())
    }
}

impl ImprovedDayStat {
    pub fn new(
        rating: f32,
        date: DateTime<Local>,
        note: &str,
        mood_tags: Vec<MoodTag>,
        activities: Vec<Activity>,
    ) -> Self {
        Self {
            rating,
            date,
            note: note.to_string(),
            mood_tags,
            activities,
        }
    }

    pub fn modify_rating(&mut self) -> &mut f32 {
        &mut self.rating
    }

    pub fn get_rating(&self) -> f32 {
        self.rating
    }

    pub fn get_date(&self) -> &DateTime<Local> {
        &self.date
    }

    pub fn get_note(&self) -> &str {
        &self.note
    }

    pub fn get_mood_tags(&self) -> &[MoodTag] {
        &self.mood_tags
    }

    pub fn get_current_time_system() -> DateTime<Local> {
        Local::now()
    }

    #[allow(dead_code)]
    pub fn get_date_time(&self) -> DateTime<Local> {
        self.date
    }

    /// Simply subtracts the two timestamps, giving you a distance the stats are apart. timestamp being a unix timestamp
    pub fn get_hour_difference(&self, compare_day_stat: &Self) -> i64 {
        (self.date.timestamp() - compare_day_stat.date.timestamp()).abs()
    }
}

#[allow(deprecated)]
impl From<DayStat> for ImprovedDayStat {
    fn from(value: DayStat) -> Self {
        let v = value.get_date_time().with_timezone(&Local);
        Self {
            rating: value.rating,
            date: v,
            note: value.note,
            mood_tags: vec![],
            activities: vec![],
        }
    }
}

#[allow(deprecated)]
impl From<ImprovedDayStat> for DayStat {
    fn from(value: ImprovedDayStat) -> Self {
        let v = value.date.timestamp();
        Self {
            rating: value.rating,
            date: v,
            note: value.note,
        }
    }
}

#[allow(deprecated)]
impl From<&ImprovedDayStat> for DayStat {
    fn from(value: &ImprovedDayStat) -> Self {
        Self {
            rating: value.rating,
            date: value.date.timestamp(),
            note: value.note.clone(),
        }
    }
}

#[allow(deprecated)]
impl From<&DayStat> for ImprovedDayStat {
    fn from(value: &DayStat) -> Self {
        Self {
            rating: value.rating,
            date: value.get_date_time().with_timezone(&Local),
            note: value.note.clone(),
            mood_tags: vec![],
            activities: vec![],
        }
    }
}
