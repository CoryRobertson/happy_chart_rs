use std::error::Error;
use vergen::EmitBuilder;

#[cfg(target_os = "windows")]
use winres::WindowsResource;

fn main() -> Result<(), Box<dyn Error>> {
    EmitBuilder::builder().all_build().all_git().emit()?;
    #[cfg(target_os = "windows")] // conditionally set icon of program on windows
    {
        WindowsResource::new()
            .set_icon("./assets/program_icon_gimp.ico")
            .compile()?;
    }
    Ok(())
}
