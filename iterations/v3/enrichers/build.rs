#[cfg(target_os = "macos")]
use std::process::Command;
use std::path::Path;

fn main() {
    // Build Swift CoreML bridge on macOS
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rerun-if-changed=../coreml-bridge/");

        // Build CoreML Bridge (optional - will use mock implementations if it fails)
        let coreml_bridge_path = Path::new("../coreml-bridge");
        if coreml_bridge_path.exists() {
            println!("Building CoreML Bridge...");
            let status = Command::new("swift")
                .args(&["build", "--configuration", "release"])
                .current_dir(coreml_bridge_path)
                .status()
                .expect("Failed to execute Swift build");

            if !status.success() {
                println!("CoreML Bridge build failed, using mock implementations");
            } else {
                println!("CoreML Bridge built successfully");
                // Note: Swift Package Manager doesn't produce traditional static libs
                // We'll use subprocess calls or dynamic linking for now
            }
        }

        // Link system frameworks for CoreML (when available)
        println!("cargo:rustc-link-lib=framework=CoreML");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Accelerate");
        println!("cargo:rustc-link-lib=framework=AVFoundation");

        // Fallback frameworks for compatibility
        println!("cargo:rustc-link-lib=framework=Speech");
        println!("cargo:rustc-link-lib=framework=Vision");
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS platforms, provide stub implementations
        println!("cargo:rustc-cfg=stub_bridges");
    }
}
