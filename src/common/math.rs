use crate::prelude::ImprovedDayStat;
use chrono::{Datelike, Weekday};

/// Calculates the x coordinate for each graph point
#[tracing::instrument]
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
#[tracing::instrument]
pub fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    distance_squared(x1, y1, x2, y2).sqrt()
}

/// Returns the coordinate point distance between two points
#[tracing::instrument]
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
