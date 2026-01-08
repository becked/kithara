<script lang="ts">
	import type { Sound } from '$lib/api';
	import { playerState, playSoundAction } from '$lib/stores/sounds.svelte';

	let { sound }: { sound: Sound } = $props();

	let isCurrentlyPlaying = $derived(
		playerState.isPlaying && playerState.currentSound?.id === sound.id
	);

	async function handleClick() {
		await playSoundAction(sound);
	}

	function formatDuration(seconds: number): string {
		const secs = Math.floor(seconds);
		return secs < 60
			? `${secs}s`
			: `${Math.floor(secs / 60)}:${(secs % 60).toString().padStart(2, '0')}`;
	}
</script>

<button
	class="sound-button"
	class:playing={isCurrentlyPlaying}
	onclick={handleClick}
	title={sound.eventName}
>
	<span class="name">{sound.displayName}</span>
	<span class="duration">{formatDuration(sound.duration)}</span>
	{#if sound.unitType}
		<span class="unit-type">{sound.unitType}</span>
	{/if}
</button>

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

	.name {
		font-weight: 500;
		color: var(--color-text);
		font-size: 0.9rem;
		line-height: 1.2;
	}

	.duration {
		font-size: 0.75rem;
		color: var(--color-text-muted);
	}

	.unit-type {
		font-size: 0.7rem;
		color: var(--color-text-muted);
		background: var(--color-bg);
		padding: 0.1rem 0.4rem;
		border-radius: var(--radius-sm);
	}

	.sound-button.playing .duration,
	.sound-button.playing .unit-type {
		color: rgba(255, 255, 255, 0.8);
	}

	.sound-button.playing .unit-type {
		background: rgba(255, 255, 255, 0.2);
	}
</style>
