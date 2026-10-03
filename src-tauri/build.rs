fn main() {
    let build_index = std::path::Path::new("../build/index.html");
    if !build_index.exists() {
        println!("cargo:warning=build/index.html not found, executing npm run build...");
        let _ = if cfg!(target_os = "windows") {
            std::process::Command::new("cmd")
                .args(["/C", "npm run build"])
                .current_dir("..")
                .status()
        } else {
            std::process::Command::new("npm")
                .arg("run")
                .arg("build")
                .current_dir("..")
                .status()
        };
    }
    tauri_build::build()
}
