//! Audio playback module using rodio with a dedicated audio thread.
//!
//! rodio's OutputStream is not Send+Sync, so we spawn a dedicated thread
//! to handle audio playback and communicate via channels.

use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Commands sent to the audio thread
enum AudioCommand {
    Play { id: String, path: PathBuf },
    Stop,
    Pause,
    Resume,
    SetVolume { volume: f32 },
    Seek { position_secs: f64 },
    GetStatus { response: Sender<AudioStatus> },
    Shutdown,
}

/// Status response from the audio thread
#[derive(Debug, Clone)]
pub struct AudioStatus {
    pub is_playing: bool,
    pub is_paused: bool,
    pub current_sound_id: Option<String>,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub volume: f32,
    pub sample_rate: u32,
    pub bitrate_kbps: u32,
}

/// Handle to communicate with the audio thread
pub struct AudioPlayer {
    command_tx: Sender<AudioCommand>,
}

impl AudioPlayer {
    /// Creates a new audio player, spawning the audio thread.
    pub fn new() -> Result<Self, String> {
        let (command_tx, command_rx) = mpsc::channel::<AudioCommand>();

        // Spawn the audio thread
        thread::spawn(move || {
            // Create the audio output on this thread (it must stay on this thread)
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(output) => output,
                Err(e) => {
                    eprintln!("Failed to create audio output: {}", e);
                    return;
                }
            };

            let mut sink: Option<Sink> = None;
            let mut current_sound_id: Option<String> = None;
            let mut current_path: Option<PathBuf> = None;
            let mut current_volume: f32 = 1.0;
            let mut duration_secs: f64 = 0.0;
            let mut sample_rate: u32 = 0;
            let mut bitrate_kbps: u32 = 0;

            // Position tracking
            let mut playback_start: Option<Instant> = None;
            let mut playback_offset: f64 = 0.0; // Position when playback started/resumed
            let mut paused_position: Option<f64> = None; // Position when paused

            // Helper to calculate current position
            let calc_position = |start: Option<Instant>, offset: f64, paused: Option<f64>| -> f64 {
                if let Some(pos) = paused {
                    return pos;
                }
                if let Some(start_time) = start {
                    return offset + start_time.elapsed().as_secs_f64();
                }
                0.0
            };

            // Process commands
            while let Ok(cmd) = command_rx.recv() {
                match cmd {
                    AudioCommand::Play { id, path } => {
                        // Stop any currently playing sound
                        if let Some(s) = sink.take() {
                            s.stop();
                        }

                        // Get file size for bitrate calculation
                        let file_size = std::fs::metadata(&path)
                            .map(|m| m.len())
                            .unwrap_or(0);

                        // Open and decode the audio file
                        match File::open(&path) {
                            Ok(file) => {
                                let reader = BufReader::new(file);
                                match Decoder::new(reader) {
                                    Ok(source) => {
                                        // Get audio properties before consuming source
                                        sample_rate = source.sample_rate();
                                        duration_secs = source.total_duration()
                                            .map(|d| d.as_secs_f64())
                                            .unwrap_or(0.0);

                                        // Calculate approximate bitrate (file_size in bytes / duration in seconds * 8 / 1000)
                                        if duration_secs > 0.0 {
                                            bitrate_kbps = ((file_size as f64 * 8.0) / (duration_secs * 1000.0)) as u32;
                                        } else {
                                            bitrate_kbps = 0;
                                        }

                                        match Sink::try_new(&stream_handle) {
                                            Ok(new_sink) => {
                                                new_sink.set_volume(current_volume);
                                                new_sink.append(source);
                                                sink = Some(new_sink);
                                                current_sound_id = Some(id);
                                                current_path = Some(path);
                                                playback_start = Some(Instant::now());
                                                playback_offset = 0.0;
                                                paused_position = None;
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to create sink: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to decode audio: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to open audio file: {}", e);
                            }
                        }
                    }
                    AudioCommand::Stop => {
                        if let Some(s) = sink.take() {
                            s.stop();
                        }
                        current_sound_id = None;
                        current_path = None;
                        playback_start = None;
                        playback_offset = 0.0;
                        paused_position = None;
                        duration_secs = 0.0;
                        sample_rate = 0;
                        bitrate_kbps = 0;
                    }
                    AudioCommand::Pause => {
                        if let Some(ref s) = sink {
                            if !s.is_paused() {
                                // Record position before pausing
                                paused_position = Some(calc_position(playback_start, playback_offset, None));
                                s.pause();
                            }
                        }
                    }
                    AudioCommand::Resume => {
                        if let Some(ref s) = sink {
                            if s.is_paused() {
                                // Resume from paused position
                                if let Some(pos) = paused_position {
                                    playback_offset = pos;
                                    playback_start = Some(Instant::now());
                                    paused_position = None;
                                }
                                s.play();
                            }
                        }
                    }
                    AudioCommand::SetVolume { volume } => {
                        current_volume = volume.clamp(0.0, 1.0);
                        if let Some(ref s) = sink {
                            s.set_volume(current_volume);
                        }
                    }
                    AudioCommand::Seek { position_secs: seek_pos } => {
                        // Seeking requires stopping current playback and starting fresh
                        if let Some(ref path) = current_path.clone() {
                            // Stop the current sink
                            if let Some(s) = sink.take() {
                                s.stop();
                            }

                            // Don't clamp to duration if duration is unknown (0)
                            let seek_pos = if duration_secs > 0.0 {
                                seek_pos.max(0.0).min(duration_secs)
                            } else {
                                seek_pos.max(0.0)
                            };

                            match File::open(&path) {
                                Ok(file) => {
                                    let reader = BufReader::new(file);
                                    match Decoder::new(reader) {
                                        Ok(source) => {
                                            // Use skip_duration for lazy seeking (doesn't decode all samples upfront)
                                            let skip_dur = Duration::from_secs_f64(seek_pos);
                                            let skipped_source = source.skip_duration(skip_dur);

                                            match Sink::try_new(&stream_handle) {
                                                Ok(new_sink) => {
                                                    new_sink.set_volume(current_volume);
                                                    new_sink.append(skipped_source);
                                                    sink = Some(new_sink);
                                                    playback_start = Some(Instant::now());
                                                    playback_offset = seek_pos;
                                                    paused_position = None;
                                                }
                                                Err(e) => {
                                                    eprintln!("Failed to create audio sink: {}", e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to decode audio for seek: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to open audio file for seek: {}", e);
                                }
                            }
                        }
                    }
                    AudioCommand::GetStatus { response } => {
                        let sink_empty = sink.as_ref().map(|s| s.empty()).unwrap_or(true);
                        let sink_paused = sink.as_ref().map(|s| s.is_paused()).unwrap_or(false);
                        let sink_len = sink.as_ref().map(|s| s.len()).unwrap_or(0);

                        // Consider "playing" if sink has content and is not paused
                        let has_sink_content = (sink_len > 0 || !sink_empty) && !sink_paused;
                        let is_playing = has_sink_content;
                        let is_paused = sink_paused || paused_position.is_some();

                        // Calculate position first (needed for clear logic)
                        let position = calc_position(playback_start, playback_offset, paused_position);

                        // Detect when track finishes playing:
                        // - Sink is truly empty (no sources queued)
                        // - Not paused
                        // - We have a current track (were playing something)
                        let track_finished = sink_empty && sink_len == 0 && !sink_paused &&
                            sink.is_some() &&
                            current_sound_id.is_some() &&
                            playback_start.is_some();  // Was actively playing

                        if track_finished {
                            // Clear playback state so position stops incrementing
                            // but keep current_sound_id so frontend knows what just finished
                            playback_start = None;
                            paused_position = None;
                        }

                        // Don't clamp position if duration is 0 (Vorbis doesn't report duration)
                        let clamped_position = if duration_secs > 0.0 {
                            position.min(duration_secs)
                        } else {
                            position
                        };

                        let _ = response.send(AudioStatus {
                            is_playing,
                            is_paused,
                            current_sound_id: current_sound_id.clone(),
                            position_secs: clamped_position,
                            duration_secs,
                            volume: current_volume,
                            sample_rate,
                            bitrate_kbps,
                        });
                    }
                    AudioCommand::Shutdown => {
                        if let Some(s) = sink.take() {
                            s.stop();
                        }
                        break;
                    }
                }
            }
        });

        Ok(Self { command_tx })
    }

    /// Plays an audio file, stopping any currently playing sound.
    pub fn play(&self, sound_id: String, file_path: PathBuf) -> Result<(), String> {
        self.command_tx
            .send(AudioCommand::Play {
                id: sound_id,
                path: file_path,
            })
            .map_err(|e| format!("Failed to send play command: {}", e))
    }

    /// Stops the currently playing sound.
    pub fn stop(&self) -> Result<(), String> {
        self.command_tx
            .send(AudioCommand::Stop)
            .map_err(|e| format!("Failed to send stop command: {}", e))
    }

    /// Pauses playback.
    pub fn pause(&self) -> Result<(), String> {
        self.command_tx
            .send(AudioCommand::Pause)
            .map_err(|e| format!("Failed to send pause command: {}", e))
    }

    /// Resumes playback after pause.
    pub fn resume(&self) -> Result<(), String> {
        self.command_tx
            .send(AudioCommand::Resume)
            .map_err(|e| format!("Failed to send resume command: {}", e))
    }

    /// Sets the playback volume (0.0 to 1.0).
    pub fn set_volume(&self, volume: f32) -> Result<(), String> {
        self.command_tx
            .send(AudioCommand::SetVolume { volume })
            .map_err(|e| format!("Failed to send volume command: {}", e))
    }

    /// Seeks to a position in seconds.
    pub fn seek(&self, position_secs: f64) -> Result<(), String> {
        self.command_tx
            .send(AudioCommand::Seek { position_secs })
            .map_err(|e| format!("Failed to send seek command: {}", e))
    }

    /// Gets the current playback status.
    pub fn get_status(&self) -> Result<AudioStatus, String> {
        let (response_tx, response_rx) = mpsc::channel();
        self.command_tx
            .send(AudioCommand::GetStatus {
                response: response_tx,
            })
            .map_err(|e| format!("Failed to send status command: {}", e))?;

        response_rx
            .recv()
            .map_err(|e| format!("Failed to receive status: {}", e))
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        let _ = self.command_tx.send(AudioCommand::Shutdown);
    }
}

/// Thread-safe wrapper for AudioPlayer, suitable for Tauri managed state.
/// Uses Arc<Mutex<>> for interior mutability, though AudioPlayer itself is thread-safe.
pub type PlayerState = Arc<AudioPlayer>;

/// Creates a new PlayerState for use with Tauri's `.manage()`.
pub fn create_player_state() -> Result<PlayerState, String> {
    Ok(Arc::new(AudioPlayer::new()?))
}
