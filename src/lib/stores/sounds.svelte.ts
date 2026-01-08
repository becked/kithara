import type { Sound, Category, UnitType } from '$lib/api';
import {
	searchSounds as apiSearchSounds,
	getCategories as apiGetCategories,
	getUnitTypes as apiGetUnitTypes,
	playSound as apiPlaySound,
	stopSound as apiStopSound,
	getPlaybackStatus
} from '$lib/api';

// Sounds state using Svelte 5 runes
export const soundsState = $state<{
	sounds: Sound[];
	categories: Category[];
	unitTypes: UnitType[];
	loading: boolean;
	error: string | null;
}>({
	sounds: [],
	categories: [],
	unitTypes: [],
	loading: false,
	error: null
});

// Filter state
export const filterState = $state<{
	query: string;
	category: string | null;
	unitType: string | null;
}>({
	query: '',
	category: null,
	unitType: null
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
 * Fetch sounds based on current filter state.
 * This is the main search function that respects all filters.
 */
export async function fetchSounds(): Promise<void> {
	soundsState.loading = true;
	soundsState.error = null;

	try {
		soundsState.sounds = await apiSearchSounds(
			filterState.query,
			filterState.category ?? undefined,
			filterState.unitType ?? undefined
		);
	} catch (error) {
		console.error('Failed to fetch sounds:', error);
		soundsState.error = `Failed to fetch sounds: ${error}`;
		soundsState.sounds = [];
	} finally {
		soundsState.loading = false;
	}
}

/**
 * Initialize the store by loading categories, unit types, and initial sounds.
 * Call this once when the app mounts.
 */
export async function initializeSounds(): Promise<void> {
	soundsState.loading = true;
	soundsState.error = null;

	try {
		// Load categories and unit types in parallel
		await Promise.all([loadCategories(), loadUnitTypes()]);

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
		playbackTimeoutId = setTimeout(async () => {
			try {
				const status = await getPlaybackStatus();
				if (!status.isPlaying) {
					playerState.isPlaying = false;
					playerState.currentSound = null;
				}
			} catch {
				// If we can't get status, assume playback finished
				playerState.isPlaying = false;
				playerState.currentSound = null;
			}
		}, durationMs);
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
