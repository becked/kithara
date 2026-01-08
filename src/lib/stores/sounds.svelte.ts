import { resolveResource } from '@tauri-apps/api/path';
import type { Sound, Category, UnitType } from '$lib/api';
import { playSound as apiPlaySound, stopSound as apiStopSound, getPlaybackStatus } from '$lib/api';

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

/**
 * Load test sounds for development/testing.
 * Uses bundled test OGG files from resources directory.
 */
export async function loadTestSounds(): Promise<void> {
	soundsState.loading = true;
	soundsState.error = null;

	try {
		// Resolve paths to bundled test resources
		const shortPath = await resolveResource('resources/test-sounds/test-short.ogg');
		const mediumPath = await resolveResource('resources/test-sounds/test-medium.ogg');
		const longPath = await resolveResource('resources/test-sounds/test-long.ogg');

		soundsState.sounds = [
			{
				id: 'test-short',
				eventName: 'Test_Short_Sound',
				displayName: 'Short Test (1s)',
				category: 'test',
				unitType: null,
				subcategory: 'test',
				duration: 1.0,
				filePath: shortPath,
				tags: ['test', 'short']
			},
			{
				id: 'test-medium',
				eventName: 'Test_Medium_Sound',
				displayName: 'Medium Test (3s)',
				category: 'test',
				unitType: 'Warrior',
				subcategory: 'test',
				duration: 3.0,
				filePath: mediumPath,
				tags: ['test', 'medium']
			},
			{
				id: 'test-long',
				eventName: 'Test_Long_Sound',
				displayName: 'Long Test (10s)',
				category: 'test',
				unitType: 'Archer',
				subcategory: 'test',
				duration: 10.0,
				filePath: longPath,
				tags: ['test', 'long']
			}
		];
	} catch (error) {
		soundsState.error = `Failed to load test sounds: ${error}`;
	} finally {
		soundsState.loading = false;
	}
}
