import type { Sound, Category, UnitType } from '$lib/api';

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
