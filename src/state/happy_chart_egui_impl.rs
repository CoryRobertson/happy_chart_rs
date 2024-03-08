use crate::common::export::export_stats_to_csv;
use crate::common::update::update_program;
use crate::common::{first_load, handle_screenshot_event};
use crate::state::error_states::HappyChartError;
use crate::state::happy_chart_state::HappyChartState;
use crate::state::tutorial_state::TutorialGoal;
use crate::ui::about_screen::draw_about_page;
use crate::ui::central_screen::{
    click_drag_zoom_detection, draw_auto_update_ui, draw_bottom_row_buttons, draw_day_lines,
    draw_stat_circles, draw_stat_line_segments, draw_stat_mouse_over_info, main_screen_button_ui,
};
use crate::ui::encryption::draw_decryption_screen;
use crate::ui::error_screen::draw_error_screen;
use crate::ui::mood_selector_menu::draw_mood_selector_screen;
use crate::ui::note_edit_screen::draw_note_edit_screen;
use crate::ui::options_menu::{
    draw_backup_settings_options_menu, draw_color_options_menu, draw_encryption_settings_menu,
    draw_graphing_options_menu, draw_stat_drawing_options_menu, options_update_thread_block,
};
use crate::ui::statistics_screen::draw_previous_duration_stats_screen;
use crate::ui::tutorial_screen::draw_tutorial_screen;
use eframe::Frame;
use egui::Context;

/// Update loop for egui
impl eframe::App for HappyChartState {
    #[tracing::instrument(skip(self, ctx, _frame))]
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if self.first_load {
            first_load(self, ctx, true);
        }

        if self.open_animation_animating {
            ctx.request_repaint();
            if self.get_day_index_animation() == self.days.len() {
                self.open_animation_animating = false;
            }
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
                draw_stat_circles(ui, self, ctx);
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

                draw_encryption_settings_menu(ui, self);

                if ui.button("Export stats to CSV").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Data", &["csv"])
                        .save_file()
                    {
                        export_stats_to_csv(path, self);
                    }
                }

                if ui.button("Restart tutorial").clicked() {
                    self.tutorial_state = TutorialGoal::BeginTutorial;
                }

                // debug save file generation, makes a pretty sine wave
                #[cfg(debug_assertions)]
                if ui.button("Generate debug day list").clicked() {
                    use crate::prelude::ImprovedDayStat;
                    use chrono::{Days, Local};
                    use std::f32::consts::PI;

                    let now = Local::now();

                    let day_count = 100;

                    let day_stats = (0..day_count)
                        .filter_map(|index| now.checked_add_days(Days::new(index)))
                        .enumerate()
                        .map(|(index, d)| {
                            (
                                ((((index as f32) / day_count as f32) * PI * 2.0).sin() * 50.0)
                                    + 50.0, // convert index into a ratio out of the length,
                                // then make it wrap around an entire cycle of sin by multiplying it by 2PI,
                                // then make it have an amplitude of 50 by multiplying it by 50, and adding 50, so it doesn't go negative
                                d,
                            )
                        })
                        .map(|(rating, date)| ImprovedDayStat::new(rating, date, "", vec![]))
                        .collect::<Vec<ImprovedDayStat>>();

                    self.days = day_stats;
                    self.program_options.x_offset = 20.0;
                    self.program_options.graph_x_scale = ((100.0 / day_count as f32) / 3.0) * 0.9;
                }

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

        if self.showing_mood_tag_selector {
            egui::Window::new("Select mood").show(ctx, |ui| {
                draw_mood_selector_screen(ctx, ui, self);
            });
        }

        if self.showing_statistics_screen {
            egui::Window::new("Stats").show(ctx, |ui| {
                draw_previous_duration_stats_screen(ctx, ui, self);
            });
        }

        if !self.error_states.is_empty() {
            if self
                .error_states
                .iter()
                .any(|err| matches!(err, HappyChartError::EncryptedSaveFile(_)))
            {
                egui::Window::new("Unlock your save file").show(ctx, |ui| {
                    if let Err(err) = draw_decryption_screen(ui, self, ctx) {
                        self.error_states.push(err);
                    }
                    if self
                        .error_states
                        .iter()
                        .any(|err| matches!(err, HappyChartError::DecryptionError(_)))
                    {
                        ui.label("Error decrypting save file, check the password.");
                    }
                });
            } else {
                egui::Window::new("An error occurred :(").show(ctx, |ui| {
                    draw_error_screen(self, ui);
                });
            }
        }

        if self.tutorial_state != TutorialGoal::TutorialClosed {
            egui::Window::new("Tutorial").show(ctx, |ui| {
                draw_tutorial_screen(ctx, ui, self);
            });
        }

        if self.note_edit_selected.is_some() {
            egui::Window::new("Note editor").show(ctx, |ui| {
                draw_note_edit_screen(ui, self);
            });
        }
    }
}
