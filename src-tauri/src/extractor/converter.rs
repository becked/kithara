//! Audio conversion pipeline: WEM -> WAV -> OGG
//! Uses vgmstream-cli and ffmpeg as Tauri sidecars.

use std::path::Path;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

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

/// Convert WEM to WAV using vgmstream-cli sidecar
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

    // Verify WAV was created
    if !wav_path.exists() {
        return Err(format!("vgmstream-cli did not create output file: {}", wav_str));
    }

    Ok(())
}

/// Convert WAV to OGG using ffmpeg sidecar
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
            "-y",        // Overwrite output without asking
            "-i", wav_str, // Input file
            "-c:a", "libvorbis", // Vorbis codec
            "-q:a", "4",  // Quality level (0-10, 4 is good balance)
            "-loglevel", "error", // Suppress verbose output
            ogg_str,     // Output file
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

    // Verify OGG was created
    if !ogg_path.exists() {
        return Err(format!("ffmpeg did not create output file: {}", ogg_str));
    }

    Ok(())
}
