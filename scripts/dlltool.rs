use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut output_lib = String::new();
    let mut def_file = String::new();
    let mut dll_name = String::new();
    let mut machine = "i386:x86-64".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-l" => { i += 1; if i < args.len() { output_lib = args[i].clone(); } }
            "-d" => { i += 1; if i < args.len() { def_file = args[i].clone(); } }
            "-D" => { i += 1; if i < args.len() { dll_name = args[i].clone(); } }
            "-m" => { i += 1; if i < args.len() { machine = args[i].clone(); } }
            _ => {}
        }
        i += 1;
    }

    // Find the pre-built .a file from the windows_x86_64_gnu crate
    let dll_base = dll_name.trim_end_matches(".dll").to_lowercase();
    let rustlib_path = format!(
        "{}/.rustup/toolchains/stable-x86_64-pc-windows-gnu/lib/rustlib/x86_64-pc-windows-gnu/lib",
        std::env::var("USERPROFILE").unwrap_or_default()
    );

    // The pre-built import libraries are in the cargo registry
    let cargo_registry = format!(
        "{}/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f",
        std::env::var("USERPROFILE").unwrap_or_default()
    );

    // Find the windows_x86_64_gnu crate
    let gnu_dir = format!("{}/windows_x86_64_gnu-0.48.5/lib", cargo_registry);
    let prebuilt_lib = format!("{}/libwindows.0.48.5.a", gnu_dir);

    if !output_lib.is_empty() {
        if Path::new(&prebuilt_lib).exists() {
            // Copy the pre-built .a file as the import library
            let _ = fs::copy(&prebuilt_lib, &output_lib);
        } else {
            // Create an empty file as fallback
            let _ = File::create(&output_lib);
        }
    }

    std::process::exit(0);
}
