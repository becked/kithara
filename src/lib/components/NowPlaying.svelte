<script lang="ts">
	import { playerState, stopSoundAction } from '$lib/stores/sounds.svelte';

	let isVisible = $derived(playerState.currentSound !== null);

	async function handleStop() {
		await stopSoundAction();
	}

</script>

{#if isVisible}
	<div class="now-playing">
		<div class="info">
			<span class="label">Now Playing</span>
			<span class="name">{playerState.currentSound?.displayName}</span>
		</div>
		<button class="stop-button" onclick={handleStop} disabled={!playerState.isPlaying}>
			Stop
		</button>
	</div>
{/if}

<style>
	.now-playing {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.75rem 1.5rem;
		background: var(--color-bg-secondary);
		border-top: 1px solid var(--color-border);
		box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.3);
		z-index: 100;
	}

	.info {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.label {
		font-size: 0.75rem;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.name {
		font-weight: 500;
		color: var(--color-primary);
	}

	.stop-button {
		padding: 0.5rem 1rem;
		background: var(--color-primary);
		color: white;
		border: none;
		border-radius: var(--radius-sm);
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s ease;
	}

	.stop-button:hover:not(:disabled) {
		background: var(--color-primary-hover);
	}

	.stop-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
