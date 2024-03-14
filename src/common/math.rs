use crate::prelude::{HappyChartState, ImprovedDayStat};
use chrono::{Datelike, Weekday};
use egui::Context;

/// Calculates the x coordinate for each graph point
pub fn improved_calculate_x(
    days: &[ImprovedDayStat],
    day: &ImprovedDayStat,
    graph_x_scale: f32,
    x_offset: f32,
) -> f32 {
    let first_day = days.first().unwrap_or(day);
    let hours: f32 = day.get_hour_difference(first_day) as f32 / 3600.0; // number of hours compared to the previous point
    let x: f32 = hours.mul_add(graph_x_scale, x_offset);
    x
}

/// Returns the coordinate point distance between two points
pub fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    distance_squared(x1, y1, x2, y2).sqrt()
}

/// Returns the coordinate point distance between two points
pub fn distance_squared(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let g1 = (x2 - x1).powi(2);
    let g2 = (y2 - y1).powi(2);
    g1 + g2
}

#[tracing::instrument]
pub fn get_average_for_day_of_week(day_of_week: Weekday, days: &[ImprovedDayStat]) -> f32 {
    let ratings = days
        .iter()
        .filter(|stat| stat.get_date().weekday() == day_of_week)
        .map(|stat| stat.get_rating())
        .collect::<Vec<f32>>();

    ratings.iter().sum::<f32>() / ratings.len() as f32
}

/// Returns a graph x scale if one can be calculated, None returned if the day list length is less than 2
#[tracing::instrument(skip(app, ctx))]
pub fn calculate_centered_graph_scaling(
    app: &HappyChartState,
    ctx: &Context,
    screen_margin: f32,
) -> Option<f32> {
    let last_day = app.days.last()?;

    if app.days.len() < 2 {
        return None;
    }

    let window_rect = ctx.screen_rect();

    // I am so very sure there is a better way to do this, but this just makes so much sense in my head.
    let final_x = improved_calculate_x(&app.days, last_day, 1.0, screen_margin);
    let target_final_x = window_rect.max.x - screen_margin;
    let frac = target_final_x / final_x;
    // app.program_options.graph_x_scale = new_scale;
    // app.program_options.x_offset = 0f32;
    Some(frac)
}
