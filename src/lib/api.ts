import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import type { Sound, Category, UnitType, ExtractionStatus, PlaybackStatus, MusicTrack } from './types';

// Re-export types for convenience
export type { Sound, Category, UnitType, ExtractionStatus, PlaybackStatus, MusicTrack };
export type { ExtractionState } from './types';

export async function searchSounds(
	query: string,
	category?: string,
	unitType?: string
): Promise<Sound[]> {
	return invoke('search_sounds', { query, category, unitType });
}

export async function getCategories(): Promise<Category[]> {
	return invoke('get_categories');
}

export async function getUnitTypes(): Promise<UnitType[]> {
	return invoke('get_unit_types');
}

export async function toggleFavorite(soundId: string): Promise<boolean> {
	return invoke('toggle_favorite', { soundId });
}

export async function getFavoritesCount(): Promise<number> {
	return invoke('get_favorites_count');
}

export async function getFavorites(): Promise<Sound[]> {
	return invoke('get_favorites');
}

export async function playSound(id: string, filePath: string): Promise<void> {
	return invoke('play_sound', { id, filePath });
}

export async function stopSound(): Promise<void> {
	return invoke('stop_sound');
}

export async function pauseSound(): Promise<void> {
	return invoke('pause_sound');
}

export async function resumeSound(): Promise<void> {
	return invoke('resume_sound');
}

export async function seekSound(positionSecs: number): Promise<void> {
	return invoke('seek_sound', { positionSecs });
}

export async function setVolume(volume: number): Promise<void> {
	return invoke('set_volume', { volume });
}

export async function getPlaybackStatus(): Promise<PlaybackStatus> {
	return invoke('get_playback_status');
}

export async function getExtractionStatus(): Promise<ExtractionStatus> {
	return invoke('get_extraction_status');
}

export async function startExtraction(gamePath: string, includeMusic: boolean = false): Promise<void> {
	return invoke('start_extraction', { gamePath, includeMusic });
}

export async function cancelExtraction(): Promise<void> {
	return invoke('cancel_extraction');
}

export async function clearCache(): Promise<void> {
	return invoke('clear_cache');
}

export async function detectGamePath(): Promise<string | null> {
	return invoke('detect_game_path');
}

export async function checkAudioDependencies(): Promise<string[]> {
	return invoke('check_audio_dependencies');
}

// ========== Music Track API ==========

export async function getMusicTracks(): Promise<MusicTrack[]> {
	return invoke('get_music_tracks');
}

export async function searchMusicTracks(query: string): Promise<MusicTrack[]> {
	return invoke('search_music_tracks', { query });
}

export async function getMusicTracksCount(): Promise<number> {
	return invoke('get_music_tracks_count');
}

// ========== Window Management ==========

// Winamp player dimensions at 2x scale
const WINAMP_WIDTH = 550 + 8;  // Player width + border
const WINAMP_HEIGHT = 232 + 350 + 8;  // Player + playlist + border

// Default soundboard dimensions
const SOUNDBOARD_WIDTH = 1200;
const SOUNDBOARD_HEIGHT = 800;

export async function setWindowForMusicPlayer(): Promise<void> {
	const window = getCurrentWindow();
	await window.setSize(new LogicalSize(WINAMP_WIDTH, WINAMP_HEIGHT));
	await window.setMinSize(new LogicalSize(WINAMP_WIDTH, 400));
	await window.setTitle('Kithara - Music Player');
	await window.center();
}

export async function setWindowForSoundboard(): Promise<void> {
	const window = getCurrentWindow();
	await window.setMinSize(new LogicalSize(800, 600));
	await window.setSize(new LogicalSize(SOUNDBOARD_WIDTH, SOUNDBOARD_HEIGHT));
	await window.setTitle('Kithara - Old World Soundboard');
	await window.center();
}
