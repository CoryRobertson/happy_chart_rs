use crate::common::get_average_for_day_of_week;
use crate::day_stats::improved_daystat::ImprovedDayStat;
use chrono::Weekday;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct StateStats {
    pub avg_weekdays: WeekdayAverages,
    pub longest_streak: Days,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Days {
    pub longest_streak: u32,
    pub streak_start_index: usize,
    pub streak_end_index: usize,
}

#[derive(Debug)]
pub struct WeekdayAverages {
    pub avg_monday: f32,
    pub avg_tuesday: f32,
    pub avg_wednesday: f32,
    pub avg_thursday: f32,
    pub avg_friday: f32,
    pub avg_saturday: f32,
    pub avg_sunday: f32,
}

impl Default for StateStats {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for WeekdayAverages {
    fn default() -> Self {
        Self::new()
    }
}

impl StateStats {
    pub fn new() -> Self {
        Self {
            avg_weekdays: WeekdayAverages::new(),
            longest_streak: Days {
                longest_streak: 0,
                streak_start_index: 0,
                streak_end_index: 0,
            },
        }
    }

    /// Calculate the longest streak present in the day stat list
    pub fn calc_streak(&mut self, list: &[ImprovedDayStat]) {
        let mut streak_start_index = 0;
        let mut streak_end_index = 0;
        let mut current_max = 0u32;

        let mut iter = 0..list.len();

        // while loop does not take ownership, or current borrowing of the iterator -> https://stackoverflow.com/questions/59045408/how-to-skip-n-items-from-inside-of-an-iterator-loop
        // using a while loop here allows us to arbitrarily skip iterator spaces when we find a new max
        while let Some(day_index) = iter.next() {
            let remaining_days = &list[day_index..];

            let mut highest = 0;
            if let Some(mut prev_day) = remaining_days.first() {
                streak_start_index = day_index;
                // iterate through each day seeing if the previous day was less than 36 hours ago, if so then increment the streak counter
                for day in remaining_days {
                    if day.date.signed_duration_since(prev_day.date).num_hours() >= 36 {
                        break;
                    }

                    highest += 1;

                    prev_day = day;
                }

                // when the streak counter is higher, we assign it to the highest streak and skip that number of elements in the iterator.
                // we skip that number of elements because the space of the streaks days will be within its streak length at least, this allows us to skip around the iterator faster.
                if highest > current_max {
                    current_max = highest;
                    iter.nth((highest - 1) as usize);
                    streak_end_index = day_index + (highest - 1) as usize;
                }
            }
        }

        self.longest_streak = Days {
            longest_streak: current_max,
            streak_start_index,
            streak_end_index,
        };
    }
}

impl WeekdayAverages {
    pub const fn new() -> Self {
        Self {
            avg_monday: 0.0,
            avg_tuesday: 0.0,
            avg_wednesday: 0.0,
            avg_thursday: 0.0,
            avg_friday: 0.0,
            avg_saturday: 0.0,
            avg_sunday: 0.0,
        }
    }

    /// Calculate all averages and set them in the stats
    pub fn calc_averages(&mut self, list: &[ImprovedDayStat]) {
        self.avg_monday = get_average_for_day_of_week(Weekday::Mon, list);
        self.avg_tuesday = get_average_for_day_of_week(Weekday::Tue, list);
        self.avg_wednesday = get_average_for_day_of_week(Weekday::Wed, list);
        self.avg_thursday = get_average_for_day_of_week(Weekday::Thu, list);
        self.avg_friday = get_average_for_day_of_week(Weekday::Fri, list);
        self.avg_saturday = get_average_for_day_of_week(Weekday::Sat, list);
        self.avg_sunday = get_average_for_day_of_week(Weekday::Sun, list);
    }
}

impl Display for Days {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.longest_streak, self.streak_start_index, self.streak_end_index
        )
    }
}
