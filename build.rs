use std::error::Error;

#[cfg(target_os = "windows")]
use winres::WindowsResource;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(target_os = "windows")] // conditionally set icon of program on windows
    {
        WindowsResource::new()
            .set_icon("./assets/program_icon_gimp.ico")
            .compile()?;
    }
    Ok(())
}
