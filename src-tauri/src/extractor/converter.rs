//! Audio conversion pipeline: WEM -> WAV -> OGG
//! Uses vgmstream-cli and ffmpeg.
//! - macOS: Sidecars for both vgmstream-cli and ffmpeg
//! - Linux: Sidecar for vgmstream-cli, system ffmpeg (apt dependency)
//! - Windows: Bundled resources (exe + DLLs)

use std::path::Path;
use tauri::AppHandle;

#[cfg(any(target_os = "macos", target_os = "linux"))]
use tauri_plugin_shell::ShellExt;

#[cfg(target_os = "windows")]
use tauri::Manager;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Convert WEM file to OGG via two-step pipeline
pub async fn convert_wem_to_ogg(
    app: &AppHandle,
    wem_path: &Path,
    ogg_path: &Path,
) -> Result<(), String> {
    // Create intermediate WAV path
    let wav_path = wem_path.with_extension("wav");

    // Step 1: WEM -> WAV using vgmstream-cli
    convert_wem_to_wav(app, wem_path, &wav_path).await?;

    // Step 2: WAV -> OGG using ffmpeg
    let result = convert_wav_to_ogg(app, &wav_path, ogg_path).await;

    // Cleanup intermediate WAV regardless of result
    let _ = std::fs::remove_file(&wav_path);

    result
}

// ============================================================================
// macOS/Linux implementation using sidecars
// ============================================================================

#[cfg(any(target_os = "macos", target_os = "linux"))]
async fn convert_wem_to_wav(
    app: &AppHandle,
    wem_path: &Path,
    wav_path: &Path,
) -> Result<(), String> {
    let wem_str = wem_path
        .to_str()
        .ok_or_else(|| "Invalid WEM path".to_string())?;
    let wav_str = wav_path
        .to_str()
        .ok_or_else(|| "Invalid WAV path".to_string())?;

    let output = app
        .shell()
        .sidecar("vgmstream-cli")
        .map_err(|e| format!("Failed to get vgmstream-cli sidecar: {}", e))?
        .args(["-o", wav_str, wem_str])
        .output()
        .await
        .map_err(|e| format!("Failed to run vgmstream-cli: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "vgmstream-cli failed (exit {}): {} {}",
            output.status.code().unwrap_or(-1),
            stderr,
            stdout
        ));
    }

    if !wav_path.exists() {
        return Err(format!(
            "vgmstream-cli did not create output file: {}",
            wav_str
        ));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
async fn convert_wav_to_ogg(
    app: &AppHandle,
    wav_path: &Path,
    ogg_path: &Path,
) -> Result<(), String> {
    let wav_str = wav_path
        .to_str()
        .ok_or_else(|| "Invalid WAV path".to_string())?;
    let ogg_str = ogg_path
        .to_str()
        .ok_or_else(|| "Invalid OGG path".to_string())?;

    let output = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("Failed to get ffmpeg sidecar: {}", e))?
        .args([
            "-y",
            "-i",
            wav_str,
            "-c:a",
            "libvorbis",
            "-q:a",
            "4",
            "-loglevel",
            "error",
            ogg_str,
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "ffmpeg failed (exit {}): {}",
            output.status.code().unwrap_or(-1),
            stderr
        ));
    }

    if !ogg_path.exists() {
        return Err(format!("ffmpeg did not create output file: {}", ogg_str));
    }

    Ok(())
}

// ============================================================================
// Linux implementation: sidecar for vgmstream, system ffmpeg
// ============================================================================

#[cfg(target_os = "linux")]
async fn convert_wav_to_ogg(
    _app: &AppHandle,
    wav_path: &Path,
    ogg_path: &Path,
) -> Result<(), String> {
    let wav_str = wav_path
        .to_str()
        .ok_or_else(|| "Invalid WAV path".to_string())?;
    let ogg_str = ogg_path
        .to_str()
        .ok_or_else(|| "Invalid OGG path".to_string())?;

    let output = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            wav_str,
            "-c:a",
            "libvorbis",
            "-q:a",
            "4",
            "-loglevel",
            "error",
            ogg_str,
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "ffmpeg failed (exit {}): {}",
            output.status.code().unwrap_or(-1),
            stderr
        ));
    }

    if !ogg_path.exists() {
        return Err(format!("ffmpeg did not create output file: {}", ogg_str));
    }

    Ok(())
}

// ============================================================================
// Windows implementation using bundled resources
// ============================================================================

#[cfg(target_os = "windows")]
async fn convert_wem_to_wav(
    app: &AppHandle,
    wem_path: &Path,
    wav_path: &Path,
) -> Result<(), String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let vgmstream_exe = resource_dir
        .join("resources-win")
        .join("vgmstream")
        .join("vgmstream-cli.exe");

    if !vgmstream_exe.exists() {
        return Err(format!(
            "vgmstream-cli.exe not found at: {}",
            vgmstream_exe.display()
        ));
    }

    let wem_str = wem_path
        .to_str()
        .ok_or_else(|| "Invalid WEM path".to_string())?;
    let wav_str = wav_path
        .to_str()
        .ok_or_else(|| "Invalid WAV path".to_string())?;

    let output = tokio::process::Command::new(&vgmstream_exe)
        .args(["-o", wav_str, wem_str])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .await
        .map_err(|e| format!("Failed to run vgmstream-cli: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "vgmstream-cli failed (exit {}): {} {}",
            output.status.code().unwrap_or(-1),
            stderr,
            stdout
        ));
    }

    if !wav_path.exists() {
        return Err(format!(
            "vgmstream-cli did not create output file: {}",
            wav_str
        ));
    }

    Ok(())
}

#[cfg(target_os = "windows")]
async fn convert_wav_to_ogg(
    app: &AppHandle,
    wav_path: &Path,
    ogg_path: &Path,
) -> Result<(), String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let ffmpeg_exe = resource_dir
        .join("resources-win")
        .join("ffmpeg")
        .join("ffmpeg.exe");

    if !ffmpeg_exe.exists() {
        return Err(format!(
            "ffmpeg.exe not found at: {}",
            ffmpeg_exe.display()
        ));
    }

    let wav_str = wav_path
        .to_str()
        .ok_or_else(|| "Invalid WAV path".to_string())?;
    let ogg_str = ogg_path
        .to_str()
        .ok_or_else(|| "Invalid OGG path".to_string())?;

    let output = tokio::process::Command::new(&ffmpeg_exe)
        .args([
            "-y",
            "-i",
            wav_str,
            "-c:a",
            "libvorbis",
            "-q:a",
            "4",
            "-loglevel",
            "error",
            ogg_str,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .await
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "ffmpeg failed (exit {}): {}",
            output.status.code().unwrap_or(-1),
            stderr
        ));
    }

    if !ogg_path.exists() {
        return Err(format!("ffmpeg did not create output file: {}", ogg_str));
    }

    Ok(())
}
