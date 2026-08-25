use std::path::PathBuf;
use std::process::Command;

const REPO: &str = "xuankew/cmdRef";

/// Get the GitHub release asset name for the current platform
fn asset_name() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "cmdref-macos-aarch64",
        ("macos", "x86_64") => "cmdref-macos-x86_64",
        ("linux", "aarch64") => "cmdref-linux-aarch64",
        ("linux", "x86_64") => "cmdref-linux-x86_64",
        ("windows", "x86_64") => "cmdref.exe",
        _ => {
            eprintln!("Unsupported platform: {}-{}", std::env::consts::OS, std::env::consts::ARCH);
            std::process::exit(1);
        }
    }
}

/// Query GitHub API for the latest release version and download URL
fn query_latest() -> Result<(String, String), String> {
    let api_url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        REPO
    );

    let output = Command::new("curl")
        .args(["-fsSL", &api_url])
        .output()
        .map_err(|e| format!("Failed to call curl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("GitHub API request failed: {}", stderr.trim()));
    }

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("JSON parse error: {}", e))?;

    let tag = json["tag_name"]
        .as_str()
        .ok_or("Missing tag_name in response")?
        .trim_start_matches('v')
        .to_string();

    let name = asset_name();
    let url = json["assets"]
        .as_array()
        .ok_or("Missing assets in response")?
        .iter()
        .find(|a| a["name"].as_str() == Some(name))
        .and_then(|a| a["browser_download_url"].as_str())
        .ok_or_else(|| format!("Asset '{}' not found in release", name))?
        .to_string();

    Ok((tag, url))
}

/// Run the update check and self-replace
pub fn run_update() {
    let current = env!("CARGO_PKG_VERSION");
    println!("cmdref {}", current);
    println!();
    println!("Checking for updates...");

    match query_latest() {
        Ok((latest, url)) => {
            if latest == current {
                println!("Already up to date! (v{})", current);
                return;
            }

            println!("New version available: v{} -> v{}", current, latest);
            println!("Downloading...");

            let tmp_dir = std::env::temp_dir();
            let tmp_file = tmp_dir.join(format!("cmdref_update_{}", std::process::id()));

            // Download with progress
            let status = Command::new("curl")
                .args(["-fL#", "-o"])
                .arg(&tmp_file)
                .arg(&url)
                .status();

            match status {
                Ok(s) if s.success() => {}
                _ => {
                    eprintln!("Download failed");
                    let _ = std::fs::remove_file(&tmp_file);
                    std::process::exit(1);
                }
            }

            println!();
            println!("Installing...");

            if let Err(e) = self_replace(&tmp_file) {
                eprintln!("Install failed: {}", e);
                let _ = std::fs::remove_file(&tmp_file);
                std::process::exit(1);
            }

            println!("Successfully updated to v{}!", latest);
        }
        Err(e) => {
            eprintln!("Update check failed: {}", e);
            std::process::exit(1);
        }
    }
}

/// Replace the current binary with the downloaded one
fn self_replace(new_binary: &PathBuf) -> Result<(), String> {
    let exe =
        std::env::current_exe().map_err(|e| format!("Cannot determine current exe path: {}", e))?;

    #[cfg(unix)]
    {
        // Unix: chmod +x, then rename over the current binary
        Command::new("chmod")
            .args(["+x", new_binary.to_str().unwrap_or("")])
            .status()
            .map_err(|e| format!("chmod failed: {}", e))?;

        std::fs::rename(new_binary, &exe)
            .map_err(|e| format!("Replace failed: {}", e))?;
    }

    #[cfg(windows)]
    {
        // Windows: can't overwrite a running exe directly
        // Rename current exe to .old, then move new binary in place
        let old = exe.with_extension("exe.old");
        let _ = std::fs::remove_file(&old);
        std::fs::rename(&exe, &old).map_err(|e| format!("Cannot rename current exe: {}", e))?;
        std::fs::rename(new_binary, &exe)
            .map_err(|e| format!("Cannot install new exe: {}", e))?;
        let _ = std::fs::remove_file(&old);
    }

    Ok(())
}
