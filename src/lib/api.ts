import { invoke } from '@tauri-apps/api/core';
import type { Sound, Category, UnitType, ExtractionStatus, PlaybackStatus } from './types';

// Re-export types for convenience
export type { Sound, Category, UnitType, ExtractionStatus, PlaybackStatus };
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

export async function getPlaybackStatus(): Promise<PlaybackStatus> {
	return invoke('get_playback_status');
}

export async function getExtractionStatus(): Promise<ExtractionStatus> {
	return invoke('get_extraction_status');
}

export async function startExtraction(gamePath: string): Promise<void> {
	return invoke('start_extraction', { gamePath });
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
