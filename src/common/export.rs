use crate::prelude::HappyChartState;
use crate::state::error_states::HappyChartError;
use std::path::PathBuf;

#[tracing::instrument(skip_all)]
pub fn export_stats_to_csv(path: PathBuf, app: &mut HappyChartState) {
    match csv::WriterBuilder::new().from_path(&path) {
        Ok(mut export_writer) => {
            app.days.iter().for_each(|day_stat| {
                let written_data = &[
                    day_stat.get_date().to_string(),
                    day_stat.get_rating().to_string(),
                    day_stat.get_note().to_string(),
                    day_stat.get_mood_tags().iter().enumerate().fold(
                        String::new(),
                        |acc, (index, mood_tag)| {
                            if index == day_stat.get_mood_tags().len() - 1 {
                                format!("{}{}", acc, mood_tag.get_text())
                            } else {
                                format!("{}{},", acc, mood_tag.get_text())
                            }
                        },
                    ),
                ];

                // println!("{:?}", written_data);

                match export_writer.write_record(written_data) {
                    Ok(_) => {}
                    Err(err) => {
                        app.error_states
                            .push(HappyChartError::ExportIO(std::io::Error::from(err), None));
                    }
                }
            });

            if let Err(export_error) = export_writer.flush() {
                app.error_states
                    .push(HappyChartError::ExportIO(export_error, Some(path)));
            }
        }
        Err(export_error) => {
            app.error_states.push(HappyChartError::ExportIO(
                std::io::Error::from(export_error),
                Some(path),
            ));
        }
    }
}
