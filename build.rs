use std::error::Error;
use vergen_git2::Emitter;
#[cfg(target_os = "windows")]
use winres::WindowsResource;

fn main() -> Result<(), Box<dyn Error>> {
    let git2 = vergen_git2::Git2Builder::all_git()?;
    let build = vergen::BuildBuilder::all_build()?;

    Emitter::new()
        .add_instructions(&git2)?
        .add_instructions(&build)?
        .emit()?;

    // EmitBuilder::builder().all_build().all_git().emit()?;
    #[cfg(target_os = "windows")] // conditionally set icon of program on windows
    {
        WindowsResource::new()
            .set_icon("./assets/program_icon_gimp.ico")
            .compile()?;
    }
    Ok(())
}
