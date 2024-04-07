use std::error::Error;
use std::fs::File;
use std::hash::{DefaultHasher, Hasher};
use std::io::Read;

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

    let walk_dir = walkdir::WalkDir::new("./src");

    let mut hasher = DefaultHasher::new();

    walk_dir
        .into_iter()
        .filter_map(|res| res.ok())
        .filter(|entry| entry.path().is_file())
        .map(|file| {
            let path = file.into_path();

            let mut file = File::open(path).unwrap();
            let mut s = String::new();

            file.read_to_string(&mut s).unwrap();

            s
        })
        .for_each(|file_content| {
            hasher.write(file_content.as_bytes());
        });

    let hash = hasher.finish();

    println!("cargo:rerun-if-changed=./src");
    println!("cargo:rustc-env=SOURCE_CODE_HASH={}", hash);

    Ok(())
}
