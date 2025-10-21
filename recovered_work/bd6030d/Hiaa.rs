#[cfg(target_os = "macos")]
use std::process::Command;
use std::path::Path;

fn main() {
    // Build Swift ASR bridge on macOS
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rerun-if-changed=../asr-bridge/");
        println!("cargo:rerun-if-changed=../vision-bridge/");

        // Build ASR Bridge
        let asr_bridge_path = Path::new("../asr-bridge");
        if asr_bridge_path.exists() {
            println!("Building ASR Bridge...");
            let status = Command::new("swift")
                .args(&["build", "--configuration", "release"])
                .current_dir(asr_bridge_path)
                .status()
                .expect("Failed to build ASR Bridge");

            if !status.success() {
                panic!("ASR Bridge build failed");
            }

            // Link the built library
            println!("cargo:rustc-link-search=native={}/build/lib", asr_bridge_path.display());
            println!("cargo:rustc-link-lib=static=ASRBridge");
        }

        // Build Vision Bridge
        let vision_bridge_path = Path::new("../vision-bridge");
        if vision_bridge_path.exists() {
            println!("Building Vision Bridge...");
            let status = Command::new("swift")
                .args(&["build", "--configuration", "release"])
                .current_dir(vision_bridge_path)
                .status()
                .expect("Failed to build Vision Bridge");

            if !status.success() {
                panic!("Vision Bridge build failed");
            }

            // Link the built library
            println!("cargo:rustc-link-search=native={}/build/lib", vision_bridge_path.display());
            println!("cargo:rustc-link-lib=static=VisionBridge");
        }

        // Link system frameworks
        println!("cargo:rustc-link-lib=framework=Speech");
        println!("cargo:rustc-link-lib=framework=Vision");
        println!("cargo:rustc-link-lib=framework=Foundation");
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS platforms, provide stub implementations
        println!("cargo:rustc-cfg=stub_bridges");
    }
}
