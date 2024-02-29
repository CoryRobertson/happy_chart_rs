#[allow(deprecated)]
use crate::day_stats::daystat::DayStat;
use chrono::{DateTime, Datelike, Local, Timelike};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ImprovedDayStat {
    pub rating: f32,
    pub date: DateTime<Local>,
    pub note: String,
}

#[allow(deprecated)]
impl From<DayStat> for ImprovedDayStat {
    #[tracing::instrument]
    fn from(value: DayStat) -> Self {
        let v = value.get_date_time().with_timezone(&Local);
        Self {
            rating: value.rating,
            date: v,
            note: value.note,
        }
    }
}

#[allow(deprecated)]
impl From<ImprovedDayStat> for DayStat {
    #[tracing::instrument]
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
    #[tracing::instrument]
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
    #[tracing::instrument]
    fn from(value: &DayStat) -> Self {
        Self {
            rating: value.rating,
            date: value.get_date_time().with_timezone(&Local),
            note: value.note.clone(),
        }
    }
}

impl Display for ImprovedDayStat {
    #[tracing::instrument(skip_all)]
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
        f.write_str(&self.note)
    }
}

impl ImprovedDayStat {
    #[tracing::instrument]
    pub fn get_current_time_system() -> DateTime<Local> {
        Local::now()
    }

    #[tracing::instrument]
    #[allow(dead_code)]
    pub fn get_date_time(&self) -> DateTime<Local> {
        self.date
    }

    /// Simply subtracts the two timestamps, giving you a distance the stats are apart. timestamp being a unix timestamp
    #[tracing::instrument]
    pub fn get_hour_difference(&self, compare_day_stat: &Self) -> i64 {
        (self.date.timestamp() - compare_day_stat.date.timestamp()).abs()
    }
}

impl Default for ImprovedDayStat {
    #[tracing::instrument]
    fn default() -> Self {
        Self {
            rating: 0.0,
            date: Local::now(),
            note: "DEFAULT NOTE".to_string(),
        }
    }
}
