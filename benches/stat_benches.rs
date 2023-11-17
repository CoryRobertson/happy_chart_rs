use chrono::{DateTime, Datelike, Local, Weekday};
use rand::{thread_rng, Rng};

fn main() {
    divan::main();
}

pub(crate) struct ImprovedDayStat {
    pub rating: f32,
    pub date: DateTime<Local>,
}

#[divan::bench]
fn day_of_week_test() -> f32 {
    fn get_average_for_day_of_week(day_of_week: Weekday, days: &[ImprovedDayStat]) -> f32 {
        let ratings = days
            .iter()
            .filter(|stat| stat.date.weekday() == day_of_week)
            .map(|stat| stat.rating)
            .collect::<Vec<f32>>();

        ratings.iter().sum::<f32>() / ratings.len() as f32
    }

    let days = {
        let mut list = vec![];
        (0..500).into_iter().for_each(|_| {
            list.push(ImprovedDayStat {
                rating: thread_rng().gen_range(0..=100) as f32,
                date: DateTime::from(
                    DateTime::from_timestamp(thread_rng().gen_range(0..=1_000_000_000), 0).unwrap(),
                ),
            });
        });
        list
    };
    get_average_for_day_of_week(Weekday::Mon, &days)
        + get_average_for_day_of_week(Weekday::Tue, &days)
        + get_average_for_day_of_week(Weekday::Wed, &days)
        + get_average_for_day_of_week(Weekday::Thu, &days)
        + get_average_for_day_of_week(Weekday::Fri, &days)
        + get_average_for_day_of_week(Weekday::Sat, &days)
        + get_average_for_day_of_week(Weekday::Sun, &days)
}
