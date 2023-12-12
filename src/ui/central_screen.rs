use crate::common::{distance, improved_calculate_x, quit, update_program};
use crate::day_stats::improved_daystat::ImprovedDayStat;
use crate::options::color_setting;
use crate::state::happy_chart_state::HappyChartState;
use crate::{BUILD_TIMESTAMP, GIT_DESCRIBE};
use chrono::Days;
use eframe::emath::{Align2, Pos2, Rect, Vec2};
use eframe::epaint::{Color32, FontId, Rounding, Stroke};
use egui::{Context, Layout, Ui, ViewportCommand};
use self_update::cargo_crate_version;

pub(crate) fn main_screen_button_ui(central_panel_ui: &mut Ui, app: &mut HappyChartState) {
    central_panel_ui.horizontal(|ui| {
        ui.label("Rating: ");
        ui.add(egui::Slider::new(&mut app.rating, 0.0..=100.0))
            .on_hover_text("The rating of the given day to be saved to the graph point.");
    });

    central_panel_ui.horizontal(|ui| {
        ui.label("Note: ");
        ui.text_edit_multiline(&mut app.note_input)
            .on_hover_text("The note to add to the next added graph point.");
    });

    if central_panel_ui.button("Add day").clicked() {
        app.days.push(ImprovedDayStat {
            rating: app.rating as f32,
            date: ImprovedDayStat::get_current_time_system(),
            note: app.note_input.clone(),
        });
        app.stats.avg_weekdays.calc_averages(&app.days);
        app.stats.calc_streak(&app.days);
        println!(
            "day added with rating {} and date {}",
            app.rating,
            ImprovedDayStat::get_current_time_system()
        );
    }

    if central_panel_ui.button("Remove day").clicked() && !app.days.is_empty() {
        app.days.remove(app.days.len() - 1);
        app.stats.avg_weekdays.calc_averages(&app.days);
    }

    central_panel_ui.horizontal(|ui| {
        ui.label("Search: ");
        ui.add_sized(
            Vec2::new(120.0, 20.0),
            egui::widgets::text_edit::TextEdit::singleline(&mut app.filter_term),
        );
    });
}

pub(crate) fn click_drag_zoom_detection(
    central_panel_ui: &mut Ui,
    app: &mut HappyChartState,
    pointer_interact_pos: Option<&Pos2>,
) {
    let within_day_lines = {
        let min_y: f32 = 220.0 - app.program_options.day_line_height_offset;
        pointer_interact_pos.map_or(false, |pos| pos.y >= min_y)
    };

    if within_day_lines {
        let right_click_down = central_panel_ui.input(|i| i.pointer.secondary_down());

        let left_click_down = central_panel_ui.input(|i| i.pointer.primary_down());

        // if right click is down, allow the xoffset to be moved
        if right_click_down {
            let drag_delta = central_panel_ui.input(|i| i.pointer.delta());

            app.program_options.x_offset += drag_delta.x;

            // if both right click and left click are down, then we allow the x scale to be changed so the user can quickly zoom into areas on the graph
            if left_click_down {
                app.program_options.graph_x_scale += -drag_delta.y / 1000.0;
                app.program_options.x_offset += drag_delta.y * (10.0);
            }

            if app.program_options.graph_x_scale.is_sign_negative() {
                app.program_options.graph_x_scale = 0.001;
            }
        }
    }
}

/// Draw the lines that represent time itself, typically 24 hours
pub(crate) fn draw_day_lines(central_panel_ui: &mut Ui, app: &mut HappyChartState) {
    if app.days.len() > 1 {
        // range for calculating how many lines in both directions on the x axis
        let range = {
            if app.program_options.x_offset > 5000.0 {
                app.program_options.x_offset as i32
            } else {
                5000
            }
        };

        for i2 in -range..range {
            // make a fake day with the first day on the list as the first day, and add 24 hours to it each time in utc time to calculate where each line goes
            let line_points: [Pos2; 2] = {
                let def = ImprovedDayStat::default();
                let d = app.days.get(0).unwrap_or(&def);

                let fake_day = ImprovedDayStat {
                    rating: 0.0,
                    date: d.date.checked_add_days(Days::new(1)).unwrap_or_default(), // fake day that starts from where the first day is, with one day added
                    note: String::new(),
                };
                let y: f32 = 220.0 - app.program_options.day_line_height_offset;
                let x = {
                    let first_day = d;
                    let hours: f32 = fake_day.get_hour_difference(first_day) as f32 / 3600.0; // number of hours compared to the previous point

                    let x: f32 = (hours * app.program_options.graph_x_scale) * i2 as f32;
                    x + app.program_options.x_offset
                };
                [Pos2::new(x, y), Pos2::new(x, 800.0)]
            };
            central_panel_ui.painter().line_segment(
                line_points,
                Stroke::new(2.0, app.program_options.color_settings.day_line_color),
            );
        }
    }
}

/// Draw the lines between each stat like a graph
pub(crate) fn draw_stat_line_segments(central_panel_ui: &mut Ui, app: &mut HappyChartState) {
    let mut prev_x = 0.0;
    let mut prev_y = 0.0;
    // draw lines loop, bottom layer
    for (i, day) in app.days.iter().enumerate() {
        let x: f32 = improved_calculate_x(
            &app.days,
            day,
            app.program_options.graph_x_scale,
            app.program_options.x_offset,
        );

        let y: f32 = day
            .rating
            .mul_add(-app.program_options.graph_y_scale, 500.0)
            - app.program_options.day_stat_height_offset;
        let points = [Pos2::new(prev_x, prev_y), Pos2::new(x, y)];

        if (prev_x != 0.0 && prev_y != 0.0) || i == 1 {
            // draw line segments connecting the dots
            central_panel_ui.painter().line_segment(
                points,
                Stroke::new(2.0, app.program_options.color_settings.line_color),
            );
        }

        prev_x = x;
        prev_y = y;
    }
}

/// draw the circled for each stat, separate color based on each stats rating
pub(crate) fn draw_stat_circles(central_panel_ui: &mut Ui, app: &mut HappyChartState) {
    let mut idx = 0;
    for day in &app.days.clone() {
        let x: f32 = improved_calculate_x(
            &app.days,
            day,
            app.program_options.graph_x_scale,
            app.program_options.x_offset,
        );
        let y: f32 = day
            .rating
            .mul_add(-app.program_options.graph_y_scale, 500.0)
            - app.program_options.day_stat_height_offset;

        let streak_color = if idx >= app.stats.longest_streak.streak_start_index
            && idx <= app.stats.longest_streak.streak_end_index && app.program_options.show_streak
        {
            app.program_options.color_settings.stat_outline_streak_color
        } else {
            app.program_options.color_settings.stat_outline_color
        };

        //draw circles on each coordinate point
        central_panel_ui.painter().circle_filled(
            Pos2::new(x, y),
            app.program_options.daystat_circle_outline_radius,
            streak_color,
        );

        let color = if !app.filter_term.is_empty() && day.note.contains(&app.filter_term) {
            Color32::LIGHT_BLUE
        } else {
            color_setting::get_shape_color_from_rating(day.rating)
        };

        central_panel_ui.painter().circle_filled(
            Pos2::new(x, y),
            app.program_options.daystat_circle_size,
            color,
        );
        idx += 1;
    }
}

/// Draw a stats info if it is moused over
pub(crate) fn draw_stat_mouse_over_info(
    central_panel_ui: &mut Ui,
    app: &mut HappyChartState,
    ctx: &Context,
) {
    let mouse_pos = ctx
        .pointer_hover_pos()
        .map_or_else(|| Pos2::new(0.0, 0.0), |a| a);
    let mut moused_over = false; // boolean used to know if we are already showing mouse over text, if so, not to render it if this is true
                                 // draw text loop, top most layer (mostly)
    for day in &app.days {
        let x: f32 = improved_calculate_x(
            &app.days,
            day,
            app.program_options.graph_x_scale,
            app.program_options.x_offset,
        );
        let y: f32 = day
            .rating
            .mul_add(-app.program_options.graph_y_scale, 500.0)
            - app.program_options.day_stat_height_offset;
        let rect_pos1 = Pos2::new(520.0, 10.0);
        let rect_pos2 = Pos2::new(770.0, 180.0);
        let text = day.to_string();

        let dist_max = app.program_options.mouse_over_radius; // maximum distance to consider a point being moused over

        if distance(mouse_pos.x, mouse_pos.y, x, y) < dist_max && !moused_over {
            // draw text near by each coordinate point
            moused_over = true;

            central_panel_ui.painter().text(
                Pos2::new(x + 20.0, y),
                Align2::LEFT_CENTER,
                &text,
                FontId::default(),
                app.program_options.color_settings.text_color, // color_setting::get_text_color(),
            );

            central_panel_ui.painter().rect_filled(
                Rect::from_two_pos(rect_pos1, rect_pos2),
                Rounding::from(20.0),
                app.program_options.color_settings.info_window_color,
            );
            central_panel_ui.style_mut().visuals.override_text_color =
                Option::from(app.program_options.color_settings.text_color);

            // info text to display in top right window
            let info_text: String = day.to_string();

            central_panel_ui.put(
                Rect::from_two_pos(rect_pos1, rect_pos2),
                egui::widgets::Label::new(&info_text),
            );
        }
    }
}

/// Draw the auto update ui on screen if needed
pub(crate) fn draw_auto_update_ui(
    central_panel_ui: &mut Ui,
    app: &mut HappyChartState,
    ctx: &Context,
) {
    if let Some(release) = &app.update_available {
        let should_show_update = match &app.auto_update_seen_version {
            None => true,
            Some(ver) => {
                self_update::version::bump_is_greater(ver, &release.version).unwrap_or(true)
            }
        };
        if should_show_update {
            if central_panel_ui.button("Dismiss update").clicked() {
                app.auto_update_seen_version = Some(release.version.to_string());
            }
            if central_panel_ui.button("Update happy chart").clicked() {
                app.update_thread.replace(Some(update_program()));
                app.auto_update_seen_version = Some(release.version.to_string());
            }
            let mid_point_x = (ctx.screen_rect().width() / 2.0) - (250.0 / 2.0);
            let quarter_point_y = ctx.screen_rect().height() / 4.0;

            central_panel_ui.painter().rect_filled(
                Rect::from_two_pos(
                    Pos2::new(mid_point_x, quarter_point_y),
                    Pos2::new(mid_point_x + 250.0, quarter_point_y + 120.0),
                ),
                Rounding::from(4.0),
                app.program_options.color_settings.info_window_color,
            );
            central_panel_ui.style_mut().visuals.override_text_color =
                Option::from(app.program_options.color_settings.text_color);

            central_panel_ui.put(
                Rect::from_two_pos(Pos2::new(mid_point_x, quarter_point_y), Pos2::new(mid_point_x + 250.0, quarter_point_y + 120.0)),
                egui::widgets::Label::new(format!("Update available:\n{}\nCurrent version:\nv{}\n\"Update happy chart\" to automagically update\nThis message will not display on next launch", release.name,cargo_crate_version!())),
            );
        }
    }
    central_panel_ui.horizontal(|ui| {
        if let Some(thread) = app.update_thread.get_mut() {
            if !thread.is_finished() {
                ui.label("Updating... ");
                ui.spinner();
            }
        }
    });
}

/// Draw the quit button as well as the options, about, and screenshot button
pub(crate) fn draw_bottom_row_buttons(
    central_panel_ui: &mut Ui,
    app: &mut HappyChartState,
    ctx: &Context,
) {
    // quit button layout
    central_panel_ui.with_layout(Layout::bottom_up(egui::Align::BOTTOM), |ui| {
        if app.starting_length != app.days.len() {
            ui.visuals_mut().override_text_color = Option::from(Color32::RED);
        } else {
            ui.style_mut().visuals.override_text_color =
                Option::from(app.program_options.color_settings.text_color);
        }

        ui.horizontal(|ui| {
            let quit_button = ui.button("Save & Quit");

            if quit_button.clicked() {
                quit(ctx, app);
            }

            ui.style_mut().visuals.override_text_color =
                Option::from(app.program_options.color_settings.text_color);

            if !app.showing_options_menu && ui.button("Options").clicked() {
                app.showing_options_menu = true;
            }

            if !app.showing_about_page && ui.button("About").clicked() {
                app.showing_about_page = true;
            }

            if ui.button("Save Screenshot").clicked() {
                // frame.request_screenshot();
                ctx.send_viewport_cmd(ViewportCommand::Screenshot);
            }

            if quit_button.hovered() {
                ui.label(egui::RichText::new(BUILD_TIMESTAMP).color(Color32::from_rgb(80, 80, 80)));
                ui.label(egui::RichText::new(GIT_DESCRIBE).color(Color32::from_rgb(80, 80, 80)));
            }
        });
    });
}
