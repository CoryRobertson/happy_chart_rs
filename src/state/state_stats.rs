use crate::common::math::get_average_for_day_of_week;
use crate::day_stats::improved_daystat::ImprovedDayStat;
use crate::state::activities::Activity;
use chrono::Weekday;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use tracing::info;

#[derive(Debug)]
pub struct StateStats {
    avg_weekdays: WeekdayAverages,
    longest_streak: Days,
    activity_stats: ActivityStats,
}

#[derive(Debug)]
pub struct ActivityStats {
    pub top_three_common_happy_activities: Vec<(Activity, u32)>,
    pub top_three_common_sad_activities: Vec<(Activity, u32)>,
    pub average_rating_for_happy_activity_days: f32,
    pub average_rating_for_sad_activity_days: f32,
    pub day_stats_counted_happy: usize,
    pub day_stats_counted_sad: usize,
}

impl ActivityStats {
    pub const fn new() -> Self {
        Self {
            top_three_common_happy_activities: vec![],
            top_three_common_sad_activities: vec![],
            average_rating_for_happy_activity_days: 0.0,
            average_rating_for_sad_activity_days: 0.0,
            day_stats_counted_happy: 0,
            day_stats_counted_sad: 0,
        }
    }

    #[tracing::instrument(skip_all)]
    fn calc_stats(&mut self, days: &[ImprovedDayStat]) {
        info!("Calculating stats for days");

        let mut day_stats_sorted_by_rating = days
            .iter()
            .filter(|day| !day.get_activities().is_empty())
            .collect::<Vec<&ImprovedDayStat>>();
        day_stats_sorted_by_rating
            .sort_by(|day1, day2| day2.get_rating().total_cmp(&day1.get_rating()));

        let day_stat_count = (days.len() as f32 * 0.25) as usize;
        let top_stats_with_activities = day_stats_sorted_by_rating
            .iter()
            .take(day_stat_count)
            .collect::<Vec<&&ImprovedDayStat>>();

        self.day_stats_counted_happy = top_stats_with_activities.len();

        let mut top_activity_map: HashMap<&Activity, u32> = HashMap::new();
        top_stats_with_activities.iter().for_each(|stat| {
            stat.get_activities()
                .iter()
                .for_each(|act| match top_activity_map.get_mut(act) {
                    None => {
                        top_activity_map.insert(act, 1);
                    }
                    Some(count) => {
                        *count += 1;
                    }
                });
        });

        let mut top_activity_list = top_activity_map
            .into_iter()
            .collect::<Vec<(&Activity, u32)>>();
        top_activity_list.sort_by_key(|(_, count)| *count);

        let bottom_stats_with_activities = day_stats_sorted_by_rating
            .iter()
            .rev()
            .take(day_stat_count)
            .collect::<Vec<&&ImprovedDayStat>>();

        self.day_stats_counted_sad = bottom_stats_with_activities.len();

        let mut bottom_activity_map: HashMap<&Activity, u32> = HashMap::new();
        bottom_stats_with_activities.iter().for_each(|stat| {
            stat.get_activities()
                .iter()
                .for_each(|act| match bottom_activity_map.get_mut(act) {
                    None => {
                        bottom_activity_map.insert(act, 1);
                    }
                    Some(count) => {
                        *count += 1;
                    }
                });
        });
        let mut bottom_activity_list = bottom_activity_map
            .into_iter()
            .collect::<Vec<(&Activity, u32)>>();
        bottom_activity_list.sort_by_key(|(_, count)| *count);

        let happy_avg_rating: f32 = top_stats_with_activities
            .iter()
            .map(|d| d.get_rating())
            .sum::<f32>()
            / top_stats_with_activities.len() as f32;
        let sad_avg_rating: f32 = bottom_stats_with_activities
            .iter()
            .map(|d| d.get_rating())
            .sum::<f32>()
            / bottom_stats_with_activities.len() as f32;

        top_activity_list.sort_by_key(|(_, c)| *c);
        top_activity_list.sort_by_key(|(a, _)| a.get_activity_name());

        bottom_activity_list.sort_by_key(|(_, c)| *c);
        bottom_activity_list.sort_by_key(|(a, _)| a.get_activity_name());

        self.top_three_common_happy_activities = top_activity_list
            .into_iter()
            .map(|(d, c)| (d.clone(), c))
            .collect();
        self.top_three_common_sad_activities = bottom_activity_list
            .into_iter()
            .rev()
            .map(|(d, c)| (d.clone(), c))
            .collect();
        self.average_rating_for_happy_activity_days = happy_avg_rating;
        self.average_rating_for_sad_activity_days = sad_avg_rating;
    }
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
    pub const fn new() -> Self {
        Self {
            avg_weekdays: WeekdayAverages::new(),
            longest_streak: Days {
                longest_streak: 0,
                streak_start_index: 0,
                streak_end_index: 0,
            },
            activity_stats: ActivityStats::new(),
        }
    }

    pub fn get_avgs_stats(&self) -> &WeekdayAverages {
        &self.avg_weekdays
    }

    pub fn get_streak_stats(&self) -> &Days {
        &self.longest_streak
    }

    pub fn get_activity_stats(&self) -> &ActivityStats {
        &self.activity_stats
    }

    #[tracing::instrument(skip_all)]
    pub fn calc_all_stats(&mut self, days: &[ImprovedDayStat], leniency: u32) {
        info!("Calculating all stats");
        self.avg_weekdays.calc_averages(days);
        self.activity_stats.calc_stats(days);
        self.calc_streak(days, leniency);
    }

    /// Calculate the longest streak present in the day stat list
    #[tracing::instrument(skip_all)]
    fn calc_streak(&mut self, list: &[ImprovedDayStat], leniency: u32) {
        info!("Calculating streak");
        let mut streak_start_index: usize = 0;
        let mut streak_end_index: usize = 0;
        let mut current_max = 0u32;

        for day_index in 0..list.len() {
            let remaining_days = &list[day_index..];

            let mut highest = 0;
            if let Some(mut prev_day) = remaining_days.first() {
                streak_start_index = {
                    let a: i32 = streak_end_index as i32 - current_max as i32;
                    // guarantee that the output start index is at least zero, so we never underflow
                    a.max(0) as usize
                };

                // iterate through each day seeing if the previous day was less than 36 hours ago, if so then increment the streak counter
                for day in remaining_days {
                    if day
                        .get_date()
                        .signed_duration_since(prev_day.get_date())
                        .num_hours()
                        > i64::from(leniency)
                    {
                        break;
                    }

                    highest += 1;

                    prev_day = day;
                }

                // when the streak counter is higher, we assign it to the highest streak and skip that number of elements in the iterator.
                if highest > current_max {
                    current_max = highest;
                    streak_end_index = day_index + (highest) as usize;
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
    #[tracing::instrument(skip_all)]
    pub fn calc_averages(&mut self, list: &[ImprovedDayStat]) {
        info!("Calculating average ratings for each day");
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
