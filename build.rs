use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn main() -> std::io::Result<()> {
    let root_path = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut src_dir = PathBuf::from(root_path.clone());
    src_dir.push("assets");
    src_dir.push("shaders");

    for entry in fs::read_dir(src_dir)? {
        if let Ok(entry) = entry {
            let path = entry.path();

            if path.extension().unwrap() != "spv" {
                run_glslc(path);
            }
        }
    }

    Ok(())
}

fn run_glslc(path: PathBuf) {
    let extension = path.extension().unwrap().to_str().unwrap();
    let output = path.with_extension(format!("{}.spv", extension));

    let output = Command::new("glslc")
        .args(&[path.to_str().unwrap(), "-o", output.to_str().unwrap()])
        .output()
        .expect("failed to run glslc");

    if !output.status.success() {
        panic!(
            "Failed to compile shader {:?}: {}\n\n{}",
            path,
            output.status,
            std::str::from_utf8(&output.stderr).unwrap()
        );
    }
}
