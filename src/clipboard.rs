#![allow(dead_code)]
use std::process::{Command, Stdio};
use std::io::Write;

/// Copy text to system clipboard (cross-platform)
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        pipe_to_command("pbcopy", &[], text)
    }

    #[cfg(target_os = "linux")]
    {
        // Try wl-copy (Wayland) first, then xclip, then xsel
        if pipe_to_command("wl-copy", &["--"], text).is_ok() {
            return Ok(());
        }
        if pipe_to_command("xclip", &["-selection", "clipboard"], text).is_ok() {
            return Ok(());
        }
        pipe_to_command("xsel", &["--clipboard", "--input"], text)
    }

    #[cfg(target_os = "windows")]
    {
        pipe_to_command("clip", &[], text)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("Unsupported platform".to_string())
    }
}

fn pipe_to_command(cmd: &str, args: &[&str], text: &str) -> Result<(), String> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| format!("{} not found", cmd))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| format!("Write failed: {}", e))?;
    }

    let status = child
        .wait()
        .map_err(|e| format!("Wait failed: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{} exited with error", cmd))
    }
}
