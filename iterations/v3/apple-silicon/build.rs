//! Build script for apple-silicon crate
//!
//! Configures torch-sys to link against the libtorch installation
//! when torch features are enabled.

use std::env;
use std::path::PathBuf;

fn main() {
    // Check if torch feature is enabled (workspace feature)
    let torch_enabled = env::var("CARGO_FEATURE_TORCH").is_ok();

    if torch_enabled {
        // Use our existing libtorch installation
        let libtorch_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("libtorch");

        if libtorch_path.exists() {
            println!("cargo:rustc-env=LIBTORCH={}", libtorch_path.display());
            println!("cargo:rustc-link-search=native={}/lib", libtorch_path.display());
            println!("cargo:rustc-link-lib=dylib=torch");
            println!("cargo:rustc-link-lib=dylib=torch_cpu");
            println!("cargo:rustc-link-lib=dylib=c10");
            println!("cargo:rerun-if-changed={}", libtorch_path.display());
            println!("cargo:warning=Using local libtorch at {}", libtorch_path.display());
        } else {
            println!("cargo:warning=libtorch not found at {}, torch features will be disabled", libtorch_path.display());
        }
    }

    // Link to BridgesFFI framework on macOS
    #[cfg(target_os = "macos")]
    {
        // The BridgesFFI framework is built by the Swift package manager
        // and should be available in the standard framework search paths
        println!("cargo:rustc-link-lib=framework=BridgesFFI");
        println!("cargo:rerun-if-changed=../bridges/Sources");
    }

    println!("cargo:rerun-if-env-changed=LIBTORCH");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_TORCH");
}



