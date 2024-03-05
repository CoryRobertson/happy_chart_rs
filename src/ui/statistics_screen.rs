use crate::day_stats::improved_daystat::ImprovedDayStat;
use crate::mood_tag::MoodTag;
use crate::prelude::HappyChartState;
use chrono::{Local, Months};
use egui::{Context, Ui};
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[tracing::instrument(skip_all)]
pub fn draw_previous_duration_stats_screen(_ctx: &Context, ui: &mut Ui, app: &mut HappyChartState) {
    if let Some(last_month) = Local::now().checked_sub_months(Months::new(1)) {
        let last_month_stat_list = app
            .days
            .iter()
            .filter(|day| day.get_date().ge(&last_month))
            .collect::<Vec<&ImprovedDayStat>>();
        // We cant show stats on a single day, since that's not very useful
        if last_month_stat_list.len() > 1 {
            let average_rating = last_month_stat_list
                .iter()
                .map(|stat| stat.get_rating())
                .sum::<f32>()
                / last_month_stat_list.len() as f32;

            ui.label(&format!(
                "Average day rating over the last month: {:.02}",
                average_rating
            ));

            // use a hashmap so we can count more easily
            let mut mood_tag_map: HashMap<MoodTag, u32> = HashMap::new();
            last_month_stat_list
                .iter()
                .map(|stat| stat.get_mood_tags())
                .for_each(|mood_tags| {
                    mood_tags
                        .iter()
                        .for_each(|mood| match mood_tag_map.get_mut(mood) {
                            None => {
                                mood_tag_map.insert(*mood, 1);
                            }
                            Some(count) => {
                                *count += 1;
                            }
                        });
                });

            // convert hashmap into a vec, so we can iterate through it in an ordered fashion
            let mut list = mood_tag_map.into_iter().collect::<Vec<(MoodTag, u32)>>();

            // two stable sorts in a row, so we don't flicker in ranking for tied mood tags, since hashmaps are unordered.
            list.sort_by(|(mood1, _), (mood2, _)| {
                let mood1_index = MoodTag::iter()
                    .enumerate()
                    .find(|(_, mood)| mood1.eq(mood))
                    .map_or(0, |(idx, _)| idx);
                let mood2_index = MoodTag::iter()
                    .enumerate()
                    .find(|(_, mood)| mood2.eq(mood))
                    .map_or(0, |(idx, _)| idx);
                mood1_index.cmp(&mood2_index)
            });
            list.sort_by(|(_, count1), (_, count2)| count2.cmp(count1));

            ui.label("Most common mood tags in the last month: ");
            for (index, (mood_tag, mood_count)) in list.iter().enumerate().take(3) {
                ui.label(&format!(
                    "{}. {} {}",
                    index + 1,
                    mood_tag.get_text(),
                    mood_count
                ));
            }
        } else {
            ui.label("There are not enough stats to show useful data yet, get charting! :)");
        }
    }

    ui.separator();
    ui.label(format!("Day stats recorded: {}", app.days.len()));
    if !app.days.is_empty() {
        ui.label(format!(
            "Average sunday: {:.0}",
            app.stats.avg_weekdays.avg_sunday
        ));
        ui.label(format!(
            "Average monday: {:.0}",
            app.stats.avg_weekdays.avg_monday
        ));
        ui.label(format!(
            "Average tuesday: {:.0}",
            app.stats.avg_weekdays.avg_tuesday
        ));
        ui.label(format!(
            "Average wednesday: {:.0}",
            app.stats.avg_weekdays.avg_wednesday
        ));
        ui.label(format!(
            "Average thursday: {:.0}",
            app.stats.avg_weekdays.avg_thursday
        ));
        ui.label(format!(
            "Average friday: {:.0}",
            app.stats.avg_weekdays.avg_friday
        ));
        ui.label(format!(
            "Average saturday: {:.0}",
            app.stats.avg_weekdays.avg_saturday
        ));
        ui.label(format!(
            "Longest streak {}",
            app.stats.longest_streak.longest_streak
        ));
        ui.label(format!(
            "Streak start-end {}-{}",
            app.stats.longest_streak.streak_start_index, app.stats.longest_streak.streak_end_index
        ));
        // TODO: heatmap using a calendar widget to show quality on each day average?
    }

    if ui.button("Close").clicked() {
        app.showing_statistics_screen = false;
    }
}
