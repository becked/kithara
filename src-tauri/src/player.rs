//! Audio playback module using rodio with a dedicated audio thread.
//!
//! rodio's OutputStream is not Send+Sync, so we spawn a dedicated thread
//! to handle audio playback and communicate via channels.

use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;

/// Commands sent to the audio thread
enum AudioCommand {
    Play { id: String, path: PathBuf },
    Stop,
    GetStatus { response: Sender<AudioStatus> },
    Shutdown,
}

/// Status response from the audio thread
#[derive(Debug, Clone)]
pub struct AudioStatus {
    pub is_playing: bool,
    pub current_sound_id: Option<String>,
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

            // Process commands
            while let Ok(cmd) = command_rx.recv() {
                match cmd {
                    AudioCommand::Play { id, path } => {
                        // Stop any currently playing sound
                        if let Some(s) = sink.take() {
                            s.stop();
                        }

                        // Open and decode the audio file
                        match File::open(&path) {
                            Ok(file) => {
                                let reader = BufReader::new(file);
                                match Decoder::new(reader) {
                                    Ok(source) => {
                                        match Sink::try_new(&stream_handle) {
                                            Ok(new_sink) => {
                                                new_sink.append(source);
                                                sink = Some(new_sink);
                                                current_sound_id = Some(id);
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
                    }
                    AudioCommand::GetStatus { response } => {
                        let is_playing = sink.as_ref().map(|s| !s.empty()).unwrap_or(false);
                        // Clear current_sound_id if playback finished
                        if !is_playing {
                            current_sound_id = None;
                        }
                        let _ = response.send(AudioStatus {
                            is_playing,
                            current_sound_id: current_sound_id.clone(),
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
