<script lang="ts">
	import { filterState } from '$lib/stores/sounds.svelte';

	// Debounce timer
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;
	const DEBOUNCE_MS = 300;

	// Local input value for immediate UI feedback
	let inputValue = $state(filterState.query);

	function handleInput(event: Event) {
		const target = event.target as HTMLInputElement;
		inputValue = target.value;

		// Clear existing timer
		if (debounceTimer) {
			clearTimeout(debounceTimer);
		}

		// Set new timer to update actual filter state
		debounceTimer = setTimeout(() => {
			filterState.query = inputValue;
		}, DEBOUNCE_MS);
	}

	function handleClear() {
		inputValue = '';
		filterState.query = '';
		if (debounceTimer) {
			clearTimeout(debounceTimer);
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			handleClear();
		}
	}
</script>

<div class="search-container">
	<svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
		<circle cx="11" cy="11" r="8" />
		<path d="m21 21-4.35-4.35" />
	</svg>
	<input
		type="text"
		class="search-input"
		placeholder="Search sounds..."
		value={inputValue}
		oninput={handleInput}
		onkeydown={handleKeydown}
	/>
	{#if inputValue}
		<button class="clear-button" onclick={handleClear} title="Clear search">
			<svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
				<path d="M18 6 6 18M6 6l12 12" />
			</svg>
		</button>
	{/if}
</div>

<style>
	.search-container {
		position: relative;
		display: flex;
		align-items: center;
	}

	.search-icon {
		position: absolute;
		left: 0.75rem;
		width: 1rem;
		height: 1rem;
		stroke-width: 2;
		color: var(--color-text-muted);
		pointer-events: none;
	}

	.search-input {
		width: 100%;
		padding: 0.625rem 2.25rem 0.625rem 2.25rem;
		background: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		color: var(--color-text);
		font-size: 0.9rem;
		outline: none;
		transition: border-color 0.15s ease;
	}

	.search-input:focus {
		border-color: var(--color-primary);
	}

	.search-input::placeholder {
		color: var(--color-text-muted);
	}

	.clear-button {
		position: absolute;
		right: 0.5rem;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.5rem;
		height: 1.5rem;
		padding: 0;
		background: transparent;
		border: none;
		border-radius: var(--radius-sm);
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.clear-button:hover {
		color: var(--color-text);
		background: var(--color-bg-tertiary);
	}

	.clear-button svg {
		width: 0.875rem;
		height: 0.875rem;
		stroke-width: 2;
	}
</style>
