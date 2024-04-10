use crate::prelude::HappyChartState;
use crate::user_feedback::FeedbackRating;
use egui::Context;
use reqwest::StatusCode;
use std::thread;
use std::time::Duration;
use strum::IntoEnumIterator;
use tracing::{error, info};

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

            ui.add_space(8.0);

            let current_feedback = app
                .ui_states
                .feedback_state
                .happy_chart_feedback
                .get_feedback();

            FeedbackRating::iter()
                .for_each(|rating| {
                    if rating == current_feedback {
                        ui.label(format!("{:?}", rating));
                    } else if ui.button(format!("{:?}", rating)).clicked() {
                        app.ui_states
                            .feedback_state
                            .happy_chart_feedback
                            .set_feedback(rating);
                    }
                });

            ui.add_space(8.0);
            ui.label("The only information that is sent is the message you type, the rating you give the program, and networking information that is always sent when ever you connect to a website. Nothing else will ever be sent.");
            ui.add_space(8.0);
            if ui.button("Send").clicked() {
                app.ui_states.feedback_state.response = None;
                let current_feedback = app.ui_states.feedback_state.happy_chart_feedback.clone();
                let request_thread = thread::spawn(move || {
                    let client = reqwest::blocking::ClientBuilder::new()
                        .timeout(Duration::from_secs(5))
                        .build()?;

                    let client_request = client
                        .post("http://localhost:8080/submit_feedback")
                        .json(&current_feedback)
                        .build()?;
                    info!("Request being sent: {:?}", client_request);
                    client.execute(client_request)
                });
                app.ui_states
                    .feedback_state
                    .submit_thread
                    .set(Some(request_thread));
            }

            if let Some(thread) = app.ui_states.feedback_state.submit_thread.take() {
                if thread.is_finished() {
                    match thread.join() {
                        Ok(joined_result) => match joined_result {
                            Ok(response) => {
                                if response.status() == StatusCode::OK {
                                    info!("Ok response: {:?}", response);
                                } else {
                                    error!("Non-Ok response: {:?}", response);
                                }
                                app.ui_states.feedback_state.response = Some(response);
                            }
                            Err(err) => {
                                error!("{}", err);
                            }
                        },
                        Err(err) => {
                            error!("{:?}", err);
                        }
                    }
                } else {
                    ui.add_space(8.0);
                    ui.spinner();
                    app.ui_states.feedback_state.submit_thread.set(Some(thread));
                }
            }

            if let Some(response_ref) = app.ui_states.feedback_state.response.as_ref() {
                ui.add_space(8.0);
                if response_ref.status() == StatusCode::OK {
                    ui.label("Successfully submit feedback to feedback server.");
                } else {
                    ui.label("Error sending feedback to feedback server.").on_hover_text(format!("{:?}",response_ref.error_for_status_ref()));
                }
            }

            ui.add_space(8.0);


            if ui.button("Close").clicked() {
                app.ui_states.feedback_state.showing_feedback_screen = false;
            }
        });
    }
}
