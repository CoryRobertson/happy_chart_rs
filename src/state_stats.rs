use crate::common::get_average_for_day_of_week;
use crate::improved_daystat::ImprovedDayStat;
use chrono::Weekday;

#[derive(Debug)]
pub struct StateStats {
    pub avg_weekdays: WeekdayAverages,
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
        }
    }
}

impl WeekdayAverages {
    pub fn new() -> Self {
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
    pub fn calc_averages(&mut self, list: &Vec<ImprovedDayStat>) {
        self.avg_monday = get_average_for_day_of_week(Weekday::Mon, list);
        self.avg_tuesday = get_average_for_day_of_week(Weekday::Tue, list);
        self.avg_wednesday = get_average_for_day_of_week(Weekday::Wed, list);
        self.avg_thursday = get_average_for_day_of_week(Weekday::Thu, list);
        self.avg_friday = get_average_for_day_of_week(Weekday::Fri, list);
        self.avg_saturday = get_average_for_day_of_week(Weekday::Sat, list);
        self.avg_sunday = get_average_for_day_of_week(Weekday::Sun, list);
    }
}
