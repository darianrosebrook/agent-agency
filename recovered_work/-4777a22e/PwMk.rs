//! Build script for apple-silicon crate
//!
//! Configures torch-sys to link against the libtorch installation
//! when torch features are enabled.

use std::env;
use std::path::PathBuf;

fn main() {
    // Check if torch feature is enabled
    let torch_enabled = env::var("CARGO_FEATURE_TORCH").is_ok();

    if torch_enabled {
        // Set LIBTORCH environment variable to point to our libtorch installation
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

            // Also set library search path for linking
            println!("cargo:rustc-link-search=native={}/lib", libtorch_path.display());

            // Link the torch libraries
            println!("cargo:rustc-link-lib=dylib=torch");
            println!("cargo:rustc-link-lib=dylib=torch_cpu");
            println!("cargo:rustc-link-lib=dylib=c10");

            println!("cargo:rerun-if-changed={}", libtorch_path.display());
        } else {
            println!("cargo:warning=libtorch not found at {}, torch features will be disabled", libtorch_path.display());
        }
    }

    // Re-run build script if libtorch directory or torch feature changes
    println!("cargo:rerun-if-env-changed=LIBTORCH");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_TORCH");
}

