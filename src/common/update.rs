use crate::prelude::HappyChartState;
use self_update::update::Release;
use self_update::{cargo_crate_version, Status};
use std::error::Error;
use std::thread;
use std::thread::JoinHandle;

#[tracing::instrument]
pub fn update_program() -> JoinHandle<Result<Status, String>> {
    thread::spawn(|| {
        match self_update::backends::github::UpdateBuilder::new()
            .repo_owner("CoryRobertson")
            .repo_name("happy_chart_rs")
            .bin_name("happy_chart_rs")
            .show_download_progress(true)
            .no_confirm(true)
            .current_version(cargo_crate_version!())
            .build()
        {
            Ok(updater) => match updater.update() {
                Ok(status) => Ok(status),
                Err(err) => Err(err.to_string()),
            },
            Err(err) => Err(err.to_string()),
        }
    })
}

#[tracing::instrument]
pub fn get_release_list() -> Result<Vec<Release>, Box<dyn Error>> {
    let list = self_update::backends::github::ReleaseList::configure()
        .repo_owner("CoryRobertson")
        .repo_name("happy_chart_rs")
        .build()?
        .fetch()?;
    #[cfg(debug_assertions)]
    println!("{:?}", list);
    Ok(list)
}

pub fn should_show_update(app: &HappyChartState) -> (bool, Option<&Release>) {
    if let Some(release) = &app.update_available {
        let should_show_update = match &app.auto_update_seen_version {
            None => true,
            Some(ver) => {
                self_update::version::bump_is_greater(ver, &release.version).unwrap_or(true)
            }
        };
        (should_show_update, Some(release))
    } else {
        (false, None)
    }
}
