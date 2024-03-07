use crate::common::color::tutorial_button_colors;
use crate::prelude::HappyChartState;
use crate::state::tutorial_state::TutorialGoal;
use egui::{Context, Ui};

#[tracing::instrument(skip_all)]
pub fn draw_tutorial_screen(ctx: &Context, ui: &mut Ui, app: &mut HappyChartState) {
    ctx.request_repaint();

    match &app.tutorial_state {
        TutorialGoal::BeginTutorial => {
            ui.heading("Welcome to happy chart!");
            ui.label("Happy chart is a multi-purpose journaling software.");
            ui.horizontal(|ui| {
                ui.label("You can find the source code to this program here:");
                ui.hyperlink("https://github.com/CoryRobertson/happy_chart_rs");
            });
            ui.label("This tutorial will cover basic usage of the program, there are many features you won't need to learn or use to make great use of the program, but feel free to explore around! :)");
            ui.horizontal(|ui| {
                ui.label("For starters, click");
                tutorial_button_colors(ui);
                if ui.button("here").clicked() {
                    app.tutorial_state = TutorialGoal::AddRating(false);
                }
                ui.label("to start the tutorial.");
            });
        }
        TutorialGoal::AddRating(added_rating) => {
            ui.heading("Add a rating to your journal entry");
            ui.label("There is a highlighted slider in the top left of the program, called rating. \
             Ratings are a subjective number from 0-100 you choose to rate your day.\
              You can be scientific with the choice of this number or just think of how you feel and pick a number from 0-100. \
              Day ratings will represent the height axis for all of your journal entries.");
            if *added_rating {
                ui.horizontal(|ui| {
                    ui.label("Click");
                    tutorial_button_colors(ui);
                    if ui.button("here").clicked() {
                        app.tutorial_state = TutorialGoal::OpenSelectMood;
                    }
                    ui.label(" to go to the next step in the tutorial.");
                });
            }
        }
        TutorialGoal::OpenSelectMood => {
            ui.heading("Open the mood menu");
            ui.label("Click the select mood button to open the mood list.");
            ui.label(
                "Moods are specific categories that a given journal entry can be categorized by,\
             they are optional to add, and should only be used if you want to. :)",
            );
        }
        TutorialGoal::SelectAMood => {
            ui.heading("Select a mood");
            ui.label("Moods are categories that can be added to a journal entry, the are not required to make an entry. \
             You can add as few or as many as you choose to, without duplicates, so a given journal entry.");
            ui.horizontal(|ui| {
                ui.label("Click");
                tutorial_button_colors(ui);
                if !app.mood_selection_list.is_empty() && ui.button("here").clicked() {
                    app.tutorial_state = TutorialGoal::WriteNote;
                }
                ui.label("once you are happy with your mood selection, to advance to the next tutorial step.");
            });
        }
        TutorialGoal::WriteNote => {
            ui.heading("Add a journal note");
            ui.label("Journal notes are text written by you that describes how you felt, your mood, what you did that day, or anything else that influenced your rating. \
            It is not required to write a note, however they are very useful when looking at past journal entries, so you can find out why you had a good or bad day.");
            ui.horizontal(|ui| {
                ui.label("Click");
                tutorial_button_colors(ui);
                if ui.button("here").clicked() {
                    app.tutorial_state = TutorialGoal::AddDay;
                }
                ui.label("once you are happy with your note, to go to the next tutorial step.");
            });
        }
        TutorialGoal::AddDay => {
            ui.heading("Add your journal entry to the program");
            ui.label("After you are satisfied with your mood selection, your rating, and your note, \
            you can go ahead and click \"Add day\" to add your entry to the list. \
            You can always remove the most recent entry by clicking \"Remove day\". All changes to entries are not saved until you click save and quit in the bottom right. \
            The save and quit button will be red if there is an entry that has been added or removed, that has not been saved.");
        }
        TutorialGoal::OpenOptions => {
            ui.heading("Open the options menu");
            ui.label("In the bottom right corner, there is an options menu, \
            it is a growing list of settings that let you customize things like visuals, graphing scale, and how the program stores its data. Go ahead and open it. :)");
        }
        TutorialGoal::DoneWithTutorial => {
            ui.heading("Thank you!");
            ui.label("This concludes the tutorial for happy chart! \
             I would absolutely love to hear about ideas or issues you have with happy chart. \
              Feel free to check out the source code, or write an issue, or suggestion at the github repository.");
            ui.hyperlink("https://github.com/CoryRobertson/happy_chart_rs");
            ui.label("I hope this program provides value to you the same it does for me!");
            ui.label("Just a few useful things to try: right click dragging on your journal entry graph,\
             right click dragging with left click held in your journal entry graph, and control left clicking on a entry.");
            ui.separator();
            tutorial_button_colors(ui);
            if ui.button("Close tutorial").clicked() {
                app.tutorial_state = TutorialGoal::TutorialClosed;
            }
        }
        TutorialGoal::TutorialClosed => {}
    }

    if !matches!(app.tutorial_state, TutorialGoal::DoneWithTutorial) {
        ui.separator();
        if ui
            .button("Skip tutorial")
            .on_hover_text("You can restart the tutorial through the options menu. :)")
            .clicked()
        {
            app.tutorial_state = TutorialGoal::TutorialClosed;
        }
    }
}
