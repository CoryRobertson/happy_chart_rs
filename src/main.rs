#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::uninlined_format_args)]

use crate::common::{
    first_load, handle_screenshot_event, read_last_session_save_file, update_program,
};
use crate::state::happy_chart_state::HappyChartState;
use crate::ui::about_screen::draw_about_page;
use crate::ui::central_screen::{
    click_drag_zoom_detection, draw_auto_update_ui, draw_bottom_row_buttons, draw_day_lines,
    draw_stat_circles, draw_stat_line_segments, draw_stat_mouse_over_info, main_screen_button_ui,
};
use crate::ui::error_screen::draw_error_screen;
use crate::ui::options_menu::{
    draw_backup_settings_options_menu, draw_color_options_menu, draw_graphing_options_menu,
    draw_stat_drawing_options_menu, options_update_thread_block,
};
use eframe::{egui, Frame, NativeOptions};
use egui::{Context, Vec2, ViewportBuilder};
#[cfg(feature = "tracing")]
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

pub(crate) mod auto_update_status;

pub(crate) mod common;
pub(crate) mod day_stats;
pub(crate) mod last_session;

pub(crate) mod options;
pub(crate) mod state;
pub(crate) mod ui;
pub(crate) const SAVE_FILE_NAME: &str = "save.ser";
pub(crate) const NEW_SAVE_FILE_NAME: &str = "happy_chart_save.ser";
pub(crate) const LAST_SESSION_FILE_NAME: &str = "happy_chart_last_session.ser";
pub(crate) const BACKUP_FILENAME_PREFIX: &str = "happy_chart_backup_";
pub(crate) const MANUAL_BACKUP_SUFFIX: &str = "_manual";
pub(crate) const BACKUP_FILE_EXTENSION: &str = "zip";
pub(crate) const GIT_DESCRIBE: &str = env!("VERGEN_GIT_DESCRIBE");
pub(crate) const BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

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

/// Update loop for egui
impl eframe::App for HappyChartState {
    #[tracing::instrument(skip(self, ctx, _frame))]
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if self.first_load {
            first_load(self, ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.input(|i| {
                for event in &i.raw.events {
                    if let egui::Event::Screenshot {
                        viewport_id: _,
                        image,
                    } = event
                    {
                        handle_screenshot_event(image);
                    }
                }
            });

            let pointer_interact_pos = ctx.pointer_interact_pos();

            main_screen_button_ui(ui, self);

            click_drag_zoom_detection(ui, self, pointer_interact_pos.as_ref());

            if self.program_options.draw_day_lines {
                draw_day_lines(ui, self, ctx);
            }

            if self.program_options.draw_daystat_lines {
                draw_stat_line_segments(ui, self);
            }

            if self.program_options.draw_daystat_circles {
                draw_stat_circles(ui, self);
            }

            draw_stat_mouse_over_info(ui, self, ctx);

            draw_auto_update_ui(ui, self, ctx);

            draw_bottom_row_buttons(ui, self, ctx);
        });

        if self.showing_options_menu {
            egui::Window::new("Options").show(ctx, |ui| {
                options_update_thread_block(ui, self);

                if ui
                    .button("Check for updates & update program")
                    .on_hover_text(self.update_status.to_text())
                    .clicked()
                {
                    self.update_thread.replace(Some(update_program()));
                }

                draw_color_options_menu(ui, self);

                draw_graphing_options_menu(ui, self);

                draw_stat_drawing_options_menu(ui, self);

                draw_backup_settings_options_menu(ui, self, ctx);

                if ui.button("Close Options Menu").clicked() {
                    self.showing_options_menu = false;
                }
            });
        }

        if self.showing_about_page {
            egui::Window::new("About").show(ctx, |ui| {
                draw_about_page(ui, self);
            });
        }

        if !self.error_states.is_empty() {
            egui::Window::new("An error occurred :(").show(ctx, |ui| {
                draw_error_screen(self, ui);
            });
        }
    }
}
