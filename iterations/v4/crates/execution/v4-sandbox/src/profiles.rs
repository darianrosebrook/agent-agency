//! macOS Sandbox Profiles
//!
//! Provides `sandbox-exec` integration for isolating tool execution on macOS.
//! Non-macOS platforms fall back to unsandboxed execution with a warning.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Available sandbox profiles for `sandbox-exec`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxProfile {
    /// Read/write within allowed path, network HTTP/HTTPS, process spawning.
    Standard,
    /// Read-only within allowed path, no network, no process spawning.
    Restricted,
}

impl SandboxProfile {
    /// Get the embedded profile content.
    pub fn content(self) -> &'static str {
        match self {
            Self::Standard => include_str!("../profiles/standard.sb"),
            Self::Restricted => include_str!("../profiles/restricted.sb"),
        }
    }

    /// Write the profile to a temporary file and return the path.
    ///
    /// `sandbox-exec -f` requires a file path, so we materialize the
    /// embedded profile to a temp location on first use.
    pub async fn materialize(self) -> Result<PathBuf, std::io::Error> {
        let name = match self {
            Self::Standard => "v4-sandbox-standard.sb",
            Self::Restricted => "v4-sandbox-restricted.sb",
        };
        let path = std::env::temp_dir().join(name);

        // Write only if missing or changed
        let needs_write = match tokio::fs::read_to_string(&path).await {
            Ok(existing) => existing != self.content(),
            Err(_) => true,
        };

        if needs_write {
            tokio::fs::write(&path, self.content()).await?;
        }

        Ok(path)
    }
}

/// Returns true if the current platform is macOS.
pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

/// Build a `tokio::process::Command` that runs the given shell command
/// under the specified sandbox profile.
///
/// On macOS: wraps with `sandbox-exec -f <profile> -D ALLOWED_PATH=<path>`.
/// On other platforms: logs a warning and falls back to unsandboxed `/bin/sh -c`.
pub async fn wrap_command_sandboxed(
    command: &str,
    profile: SandboxProfile,
    allowed_path: &str,
) -> Result<tokio::process::Command, SandboxProfileError> {
    if !is_macos() {
        tracing::warn!(
            profile = ?profile,
            "sandbox-exec not available on this platform, falling back to unsandboxed execution"
        );
        let mut cmd = tokio::process::Command::new("/bin/sh");
        cmd.arg("-c").arg(command);
        return Ok(cmd);
    }

    let profile_path = profile
        .materialize()
        .await
        .map_err(|e| SandboxProfileError::IoError(e.to_string()))?;

    let mut cmd = tokio::process::Command::new("sandbox-exec");
    cmd.arg("-f")
        .arg(&profile_path)
        .arg("-D")
        .arg(format!("ALLOWED_PATH={allowed_path}"))
        .arg("/bin/sh")
        .arg("-c")
        .arg(command);

    Ok(cmd)
}

/// Errors from sandbox profile operations.
#[derive(Debug, thiserror::Error)]
pub enum SandboxProfileError {
    #[error("IO error: {0}")]
    IoError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_content_not_empty() {
        let standard = SandboxProfile::Standard.content();
        assert!(standard.contains("(version 1)"));
        assert!(standard.contains("ALLOWED_PATH"));
        assert!(standard.contains("(allow network-outbound"));
        assert!(standard.contains("(allow file-write*"));

        let restricted = SandboxProfile::Restricted.content();
        assert!(restricted.contains("(version 1)"));
        // Restricted should NOT have network-outbound or file-write allow rules
        assert!(!restricted.contains("(allow network-outbound"));
        assert!(!restricted.contains("(allow file-write*"));
    }

    #[tokio::test]
    async fn test_profile_materialize() {
        let path = SandboxProfile::Standard.materialize().await.unwrap();
        assert!(path.exists());

        let content = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(content.contains("(version 1)"));
    }

    #[tokio::test]
    async fn test_wrap_command_produces_valid_command() {
        let cmd = wrap_command_sandboxed("echo hello", SandboxProfile::Standard, "/tmp/test")
            .await
            .unwrap();

        let program = cmd.as_std().get_program().to_string_lossy().to_string();

        if is_macos() {
            assert_eq!(program, "sandbox-exec");
            let args: Vec<_> = cmd
                .as_std()
                .get_args()
                .map(|a| a.to_string_lossy().to_string())
                .collect();
            assert!(args.contains(&"-f".to_string()));
            assert!(args.contains(&"ALLOWED_PATH=/tmp/test".to_string()));
            assert!(args.contains(&"echo hello".to_string()));
        } else {
            assert_eq!(program, "/bin/sh");
        }
    }

    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn test_sandboxed_command_succeeds() {
        let mut cmd =
            wrap_command_sandboxed("echo hello", SandboxProfile::Standard, "/tmp")
                .await
                .unwrap();

        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let output = cmd.output().await.unwrap();
        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.trim().contains("hello"));
    }

    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn test_restricted_blocks_network() {
        let mut cmd = wrap_command_sandboxed(
            "curl -s --max-time 2 http://example.com",
            SandboxProfile::Restricted,
            "/tmp",
        )
        .await
        .unwrap();

        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let output = cmd.output().await.unwrap();
        // Under restricted profile, network should be blocked
        assert!(
            !output.status.success(),
            "curl should fail under restricted sandbox"
        );
    }
}
