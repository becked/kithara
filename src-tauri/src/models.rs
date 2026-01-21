use serde::{Deserialize, Serialize};

#[cfg(test)]
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export, export_to = "../../src/lib/types/"))]
#[serde(rename_all = "camelCase")]
pub struct Sound {
    pub id: String,
    pub event_name: String,
    pub display_name: String,
    pub category: String,
    pub unit_type: Option<String>,
    pub subcategory: String,
    pub duration: f64,
    pub file_path: String,
    pub tags: Vec<String>,
    pub is_favorite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export, export_to = "../../src/lib/types/"))]
pub struct Category {
    pub id: String,
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export, export_to = "../../src/lib/types/"))]
pub struct UnitType {
    pub id: String,
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export, export_to = "../../src/lib/types/"))]
#[serde(rename_all = "snake_case")]
pub enum ExtractionState {
    NotStarted,
    InProgress,
    Complete,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export, export_to = "../../src/lib/types/"))]
#[serde(rename_all = "camelCase")]
pub struct ExtractionStatus {
    pub state: ExtractionState,
    pub progress: f32,
    pub current_file: Option<String>,
    pub error: Option<String>,
}

impl Default for ExtractionStatus {
    fn default() -> Self {
        Self {
            state: ExtractionState::NotStarted,
            progress: 0.0,
            current_file: None,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export, export_to = "../../src/lib/types/"))]
#[serde(rename_all = "camelCase")]
pub struct PlaybackStatus {
    pub is_playing: bool,
    pub is_paused: bool,
    pub current_sound_id: Option<String>,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub volume: f32,
    pub sample_rate: u32,
    pub bitrate_kbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export, export_to = "../../src/lib/types/"))]
#[serde(rename_all = "camelCase")]
pub struct MusicTrack {
    pub id: String,
    pub title: String,
    pub file_path: String,
    pub duration_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export, export_to = "../../src/lib/types/"))]
#[serde(rename_all = "camelCase")]
pub struct ExtractionOptions {
    pub include_sounds: bool,
    pub include_music: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_typescript_bindings() {
        Sound::export_all().expect("Failed to export Sound");
        Category::export_all().expect("Failed to export Category");
        UnitType::export_all().expect("Failed to export UnitType");
        ExtractionState::export_all().expect("Failed to export ExtractionState");
        ExtractionStatus::export_all().expect("Failed to export ExtractionStatus");
        PlaybackStatus::export_all().expect("Failed to export PlaybackStatus");
        MusicTrack::export_all().expect("Failed to export MusicTrack");
        ExtractionOptions::export_all().expect("Failed to export ExtractionOptions");
    }
}
