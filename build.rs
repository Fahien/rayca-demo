// This build step compiles the shaders from Slang to SPIR-V format.
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo::warning={}", format!($($tokens)*))
    }
}

fn main() {
    // Get the current directory
    let cargo_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let shaders_path = cargo_dir.join("shaders");

    // Iterate shaders directory
    let shaders_dir = std::fs::read_dir(&shaders_path).expect("Failed to read shaders directory");
    for entry in shaders_dir {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.extension() == Some(std::ffi::OsStr::new("slang")) {
            // Compile the Slang shader to SPIR-V
            // remove the ".slang" extension and replace it with ".spv"
            let path_str = path.to_str().unwrap().replace("slang", "spv");
            let output_path =
                PathBuf::from_str(&path_str).expect("Failed to convert path to string");
            p!("Compiling shader: {:?} to {:?}", path, output_path);

            Command::new("slangc")
                .arg(path)
                .arg("-target")
                .arg("spirv")
                .arg("-profile")
                .arg("sm_4_0")
                .arg("-o")
                .arg(output_path)
                .status()
                .expect("Failed to compile shader");
        }
    }

    // Print cargo instructions to link the generated files
    //println!("cargo:rerun-if-changed=shaders/");
}
