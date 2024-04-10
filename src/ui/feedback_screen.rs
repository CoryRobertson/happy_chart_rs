use crate::prelude::HappyChartState;
use crate::user_feedback::FeedbackRating;
use egui::Context;
use std::thread;
use std::time::Duration;
use strum::IntoEnumIterator;
use tracing::debug;

pub fn draw_feedback_window(ctx: &Context, app: &mut HappyChartState) {
    if app.ui_states.feedback_state.showing_feedback_screen {
        egui::Window::new("Feedback").show(ctx, |ui| {
            ui.heading("Feedback heading");
            ui.label("Feedback message:");
            ui.text_edit_multiline(
                app.ui_states
                    .feedback_state
                    .happy_chart_feedback
                    .get_message_mut(),
            );

            let current_feedback = app
                .ui_states
                .feedback_state
                .happy_chart_feedback
                .get_feedback();

            FeedbackRating::iter()
                .into_iter()
                .filter(|rating| *rating != current_feedback)
                .for_each(|rating| {
                    if ui.button(format!("{:?}", rating)).clicked() {
                        app.ui_states
                            .feedback_state
                            .happy_chart_feedback
                            .set_feedback(rating);
                    }
                });

            if ui.button("Send").clicked() {
                let current_feedback = app.ui_states.feedback_state.happy_chart_feedback.clone();
                let request_thread = thread::spawn(move || {
                    if let Ok(client) = reqwest::blocking::ClientBuilder::new()
                        .timeout(Duration::from_secs(5))
                        .build()
                    {
                        let thing = client
                            .post("localhost:8080/submit_feedback")
                            .json(&current_feedback)
                            .build()
                            .unwrap();
                        debug!("{:?}", thing);
                        let response = client.execute(thing).unwrap();
                        debug!("{:?}", response);
                    }
                });
                app.ui_states
                    .feedback_state
                    .submit_thread
                    .set(Some(request_thread));
            }

            if ui.button("Close").clicked() {
                app.ui_states.feedback_state.showing_feedback_screen = false;
            }
        });
    }
}
