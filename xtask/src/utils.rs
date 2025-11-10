use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;
use tracing::debug;

/// Run a command and return success/failure
pub async fn run_command(program: &str, args: &[&str]) -> Result<()> {
    debug!("Running: {} {}", program, args.join(" "));

    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = cmd.status().await?;

    if !status.success() {
        anyhow::bail!("Command failed with exit code: {}", status.code().unwrap_or(-1));
    }

    Ok(())
}

/// Run a command and capture its output
#[allow(dead_code)]
pub async fn capture_command_output(program: &str, args: &[&str]) -> Result<String> {
    debug!("Capturing output from: {} {}", program, args.join(" "));

    let output = Command::new(program)
        .args(args)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed: {}", stderr);
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.trim().to_string())
}

/// Check if a command exists on the system
#[allow(dead_code)]
pub async fn command_exists(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get the project root directory
#[allow(dead_code)]
pub fn project_root() -> Result<std::path::PathBuf> {
    let mut current = std::env::current_dir()?;
    loop {
        if current.join("Cargo.toml").exists() {
            return Ok(current);
        }
        if !current.pop() {
            break;
        }
    }
    anyhow::bail!("Could not find project root (no Cargo.toml found)");
}

/// Find all Rust crates in the workspace
#[allow(dead_code)]
pub fn find_rust_crates() -> Result<Vec<std::path::PathBuf>> {
    let root = project_root()?;
    let mut crates = Vec::new();

    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "Cargo.toml" {
            if let Some(parent) = entry.path().parent() {
                crates.push(parent.to_path_buf());
            }
        }
    }

    Ok(crates)
}


