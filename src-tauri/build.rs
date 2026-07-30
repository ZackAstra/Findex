fn main() {
    // Attempt Tauri build with resource compilation
    // If it fails (e.g., missing windres), continue without resources
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tauri_build::build();
    }));
    
    match result {
        Ok(()) => {
            println!("cargo:warning=Tauri build completed successfully");
        }
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown error".to_string()
            };
            println!("cargo:warning=Tauri build script warning (non-fatal): {}", msg);
            println!("cargo:warning=Continuing without Windows resource compilation");
        }
    }
}
