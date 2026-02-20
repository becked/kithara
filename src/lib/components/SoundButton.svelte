<script lang="ts">
	import type { Sound } from '$lib/api';
	import { playerState, playSoundAction, toggleFavoriteAction } from '$lib/stores/sounds.svelte';

	let { sound }: { sound: Sound } = $props();

	let isCurrentlyPlaying = $derived(
		playerState.isPlaying && playerState.currentSound?.id === sound.id
	);

	async function handleClick() {
		await playSoundAction(sound);
	}

	async function handleFavoriteClick(e: MouseEvent) {
		e.stopPropagation();
		await toggleFavoriteAction(sound.id);
	}

</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="sound-button"
	class:playing={isCurrentlyPlaying}
	onclick={handleClick}
	onkeydown={(e) => e.key === 'Enter' && handleClick()}
	role="button"
	tabindex="0"
	title={sound.eventName}
>
	<div class="button-header">
		<span class="name">{sound.displayName}</span>
		<button
			class="favorite-btn"
			class:favorited={sound.isFavorite}
			onclick={handleFavoriteClick}
			title={sound.isFavorite ? 'Remove from favorites' : 'Add to favorites'}
		>
			<svg
				viewBox="0 0 24 24"
				width="14"
				height="14"
				fill={sound.isFavorite ? 'currentColor' : 'none'}
				stroke="currentColor"
				stroke-width="2"
			>
				<path
					d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"
				/>
			</svg>
		</button>
	</div>
	{#if sound.unitType}
		<span class="unit-type">{sound.unitType}</span>
	{/if}
</div>

<style>
	.sound-button {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 0.25rem;
		padding: 0.75rem;
		background: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		text-align: left;
		cursor: pointer;
		transition: all 0.15s ease;
		min-height: 70px;
		width: 100%;
	}

	.sound-button:hover {
		background: var(--color-bg-tertiary);
		border-color: var(--color-primary);
	}

	.sound-button.playing {
		background: var(--color-primary);
		border-color: var(--color-primary);
	}

	.sound-button.playing .name {
		color: white;
	}

	.button-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		width: 100%;
		gap: 0.5rem;
	}

	.name {
		font-weight: 500;
		color: var(--color-text);
		font-size: 0.9rem;
		line-height: 1.2;
		flex: 1;
	}

	.favorite-btn {
		flex-shrink: 0;
		padding: 0.25rem;
		background: transparent;
		border: none;
		border-radius: var(--radius-sm);
		color: var(--color-text-muted);
		cursor: pointer;
		transition: color 0.15s ease;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.favorite-btn:hover {
		color: var(--color-primary);
	}

	.favorite-btn.favorited {
		color: var(--color-primary);
	}

	.sound-button.playing .favorite-btn {
		color: rgba(255, 255, 255, 0.7);
	}

	.sound-button.playing .favorite-btn:hover,
	.sound-button.playing .favorite-btn.favorited {
		color: white;
	}

	.unit-type {
		font-size: 0.7rem;
		color: var(--color-text-muted);
		background: var(--color-bg);
		padding: 0.1rem 0.4rem;
		border-radius: var(--radius-sm);
	}

	.sound-button.playing .unit-type {
		color: rgba(255, 255, 255, 0.8);
	}

	.sound-button.playing .unit-type {
		background: rgba(255, 255, 255, 0.2);
	}
</style>
