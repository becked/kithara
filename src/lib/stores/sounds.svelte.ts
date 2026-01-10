import type { Sound, Category, UnitType } from '$lib/api';
import {
	searchSounds as apiSearchSounds,
	getCategories as apiGetCategories,
	getUnitTypes as apiGetUnitTypes,
	playSound as apiPlaySound,
	stopSound as apiStopSound,
	getPlaybackStatus,
	toggleFavorite as apiToggleFavorite,
	getFavoritesCount as apiGetFavoritesCount,
	getFavorites as apiGetFavorites
} from '$lib/api';

// Sounds state using Svelte 5 runes
export const soundsState = $state<{
	sounds: Sound[];
	categories: Category[];
	unitTypes: UnitType[];
	favoritesCount: number;
	loading: boolean;
	error: string | null;
}>({
	sounds: [],
	categories: [],
	unitTypes: [],
	favoritesCount: 0,
	loading: false,
	error: null
});

// Filter state
export const filterState = $state<{
	query: string;
	category: string | null;
	unitType: string | null;
	showFavoritesOnly: boolean;
}>({
	query: '',
	category: null,
	unitType: null,
	showFavoritesOnly: false
});

// Player state
export const playerState = $state<{
	currentSound: Sound | null;
	isPlaying: boolean;
}>({
	currentSound: null,
	isPlaying: false
});

// Playback timeout handle for checking when sound finishes
let playbackTimeoutId: ReturnType<typeof setTimeout> | null = null;

/**
 * Schedule a check for playback completion.
 * If audio is still playing when checked, reschedules another check.
 */
function schedulePlaybackCheck(delayMs: number): void {
	playbackTimeoutId = setTimeout(async () => {
		try {
			const status = await getPlaybackStatus();
			if (!status.isPlaying) {
				playerState.isPlaying = false;
				playerState.currentSound = null;
			} else {
				// Audio still playing - check again shortly
				schedulePlaybackCheck(100);
			}
		} catch {
			// If we can't get status, assume playback finished
			playerState.isPlaying = false;
			playerState.currentSound = null;
		}
	}, delayMs);
}

/**
 * Load categories from the backend.
 */
export async function loadCategories(): Promise<void> {
	try {
		soundsState.categories = await apiGetCategories();
	} catch (error) {
		console.error('Failed to load categories:', error);
		soundsState.error = `Failed to load categories: ${error}`;
	}
}

/**
 * Load unit types from the backend.
 */
export async function loadUnitTypes(): Promise<void> {
	try {
		soundsState.unitTypes = await apiGetUnitTypes();
	} catch (error) {
		console.error('Failed to load unit types:', error);
		soundsState.error = `Failed to load unit types: ${error}`;
	}
}

/**
 * Load favorites count from the backend.
 */
export async function loadFavoritesCount(): Promise<void> {
	try {
		soundsState.favoritesCount = await apiGetFavoritesCount();
	} catch (error) {
		console.error('Failed to load favorites count:', error);
	}
}

/**
 * Fetch sounds based on current filter state.
 * This is the main search function that respects all filters.
 */
export async function fetchSounds(): Promise<void> {
	soundsState.loading = true;
	soundsState.error = null;

	try {
		if (filterState.showFavoritesOnly) {
			// Fetch all favorites from the backend
			soundsState.sounds = await apiGetFavorites();
		} else {
			soundsState.sounds = await apiSearchSounds(
				filterState.query,
				filterState.category ?? undefined,
				filterState.unitType ?? undefined
			);
		}
	} catch (error) {
		console.error('Failed to fetch sounds:', error);
		soundsState.error = `Failed to fetch sounds: ${error}`;
		soundsState.sounds = [];
	} finally {
		soundsState.loading = false;
	}
}

/**
 * Initialize the store by loading categories, unit types, favorites count, and initial sounds.
 * Call this once when the app mounts.
 */
export async function initializeSounds(): Promise<void> {
	soundsState.loading = true;
	soundsState.error = null;

	try {
		// Load categories, unit types, and favorites count in parallel
		await Promise.all([loadCategories(), loadUnitTypes(), loadFavoritesCount()]);

		// Then fetch initial sounds
		await fetchSounds();
	} catch (error) {
		soundsState.error = `Failed to initialize: ${error}`;
	} finally {
		soundsState.loading = false;
	}
}

/**
 * Play a sound, stopping any currently playing sound.
 */
export async function playSoundAction(sound: Sound): Promise<void> {
	// Clear any existing timeout
	if (playbackTimeoutId) {
		clearTimeout(playbackTimeoutId);
		playbackTimeoutId = null;
	}

	// Update state optimistically
	playerState.currentSound = sound;
	playerState.isPlaying = true;

	try {
		await apiPlaySound(sound.id, sound.filePath);

		// Set timer to check when playback should be done
		// Add 200ms buffer to account for timing differences
		const durationMs = sound.duration * 1000 + 200;
		schedulePlaybackCheck(durationMs);
	} catch (error) {
		playerState.isPlaying = false;
		playerState.currentSound = null;
		soundsState.error = `Failed to play sound: ${error}`;
	}
}

/**
 * Stop the currently playing sound.
 */
export async function stopSoundAction(): Promise<void> {
	// Clear any pending timeout
	if (playbackTimeoutId) {
		clearTimeout(playbackTimeoutId);
		playbackTimeoutId = null;
	}

	// Update state immediately
	playerState.isPlaying = false;
	playerState.currentSound = null;

	try {
		await apiStopSound();
	} catch (error) {
		soundsState.error = `Failed to stop sound: ${error}`;
	}
}

/**
 * Toggle favorite status for a sound.
 * Updates both backend and local state optimistically.
 */
export async function toggleFavoriteAction(soundId: string): Promise<void> {
	// Find the sound in the current list
	const soundIndex = soundsState.sounds.findIndex((s) => s.id === soundId);
	if (soundIndex === -1) return;

	// Optimistically update local state
	const previousState = soundsState.sounds[soundIndex].isFavorite;
	const previousCount = soundsState.favoritesCount;
	soundsState.sounds[soundIndex].isFavorite = !previousState;
	soundsState.favoritesCount += previousState ? -1 : 1;

	try {
		await apiToggleFavorite(soundId);
	} catch (error) {
		// Revert on failure
		soundsState.sounds[soundIndex].isFavorite = previousState;
		soundsState.favoritesCount = previousCount;
		soundsState.error = `Failed to toggle favorite: ${error}`;
	}
}
