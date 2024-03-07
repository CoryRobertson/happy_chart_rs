use crate::common::color::{
    get_tutorial_highlight_glowing_color, get_tutorial_lowlight_glowing_color,
};
use crate::common::mood_tag::MoodTag;
use crate::prelude::HappyChartState;
use crate::state::tutorial_state::TutorialGoal;
use egui::scroll_area::ScrollBarVisibility;
use egui::{Context, ScrollArea, Ui};
use strum::IntoEnumIterator;

#[tracing::instrument(skip_all)]
pub fn draw_mood_selector_screen(_ctx: &Context, ui: &mut Ui, app: &mut HappyChartState) {
    if !app.mood_selection_list.is_empty() {
        ui.label("Selected moods: ");
        egui::Grid::new("selected mood grid").show(ui, |ui| {
            let row_width = 4;
            let iteration_list = app
                .mood_selection_list
                .clone()
                .iter()
                .copied()
                .enumerate()
                .collect::<Vec<(usize, MoodTag)>>();
            for (index, mood) in iteration_list {
                if ui.button(mood.get_text()).clicked() {
                    app.mood_selection_list
                        .retain(|search_mood| *search_mood != mood);
                }

                if index != 0 && index % row_width == (row_width - 1) {
                    ui.end_row();
                    ui.end_row();
                }
            }
        });

        ui.separator();
    }

    ui.label("Add moods:");

    let old_widget_visuals = ui.style().visuals.widgets.inactive;

    if app.mood_selection_list.is_empty() && matches!(app.tutorial_state, TutorialGoal::SelectAMood)
    {
        let mut modified_widget_visuals = ui.style().visuals.widgets.inactive;
        modified_widget_visuals.bg_fill = get_tutorial_highlight_glowing_color(0);
        modified_widget_visuals.fg_stroke.color = get_tutorial_highlight_glowing_color(2);
        modified_widget_visuals.weak_bg_fill = get_tutorial_lowlight_glowing_color(0);
        ui.style_mut().visuals.widgets.inactive = modified_widget_visuals;
    }
    ScrollArea::vertical()
        .enable_scrolling(true)
        .auto_shrink(true)
        .drag_to_scroll(true)
        .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
        .max_height(400.0)
        .show(ui, |ui| {
            egui::Grid::new("mood selection grid").show(ui, |ui| {
                let row_width = 4;

                let mood_iteration_list = MoodTag::iter()
                    .enumerate()
                    .collect::<Vec<(usize, MoodTag)>>();

                for (index, mood) in mood_iteration_list {
                    if app.mood_selection_list.contains(&mood) {
                        ui.label(mood.get_text());
                    } else if ui.button(&mood.get_text()).clicked() {
                        app.mood_selection_list.push(mood);
                        app.mood_selection_list.dedup();
                    }

                    if index != 0 && index % row_width == (row_width - 1) {
                        ui.end_row();
                        ui.end_row();
                    }
                }
            });
        });

    ui.style_mut().visuals.widgets.inactive = old_widget_visuals;

    ui.separator();

    if !app.mood_selection_list.is_empty() && ui.button("Clear mood list").clicked() {
        app.mood_selection_list.clear();
    }
    if ui.button("Close").clicked() {
        app.showing_mood_tag_selector = false;
    }
}
