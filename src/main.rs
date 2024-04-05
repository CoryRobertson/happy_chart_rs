#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::uninlined_format_args)]

use eframe::NativeOptions;
use egui::{Vec2, ViewportBuilder};
use happy_chart_rs::prelude::{read_last_session_save_file, HappyChartState};

#[tracing::instrument]
fn main() {
    #[cfg(feature = "tracing")]
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default()),
    )
    .expect("Unable to setup tracy layer");

    let window_size: Vec2 = read_last_session_save_file().window_size.into();

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size(window_size),
        ..Default::default()
    };

    eframe::run_native(
        "Happy Chart",
        native_options,
        Box::new(|cc| Box::new(HappyChartState::new(cc))),
    )
    .expect("Failed to run egui app");
}
