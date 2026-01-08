<script lang="ts">
	import { onMount } from 'svelte';
	import SoundGrid from '$lib/components/SoundGrid.svelte';
	import NowPlaying from '$lib/components/NowPlaying.svelte';
	import { soundsState, loadTestSounds } from '$lib/stores/sounds.svelte';

	let tauriAvailable = $state(false);

	onMount(async () => {
		// Check if we're running in Tauri
		if ('__TAURI__' in window) {
			tauriAvailable = true;
			await loadTestSounds();
		}
	});
</script>

<main>
	<header>
		<h1>Kithara</h1>
		<p class="subtitle">Old World Soundboard</p>
	</header>

	{#if soundsState.loading}
		<div class="loading">Loading sounds...</div>
	{:else if soundsState.error}
		<div class="error">{soundsState.error}</div>
	{:else if !tauriAvailable}
		<div class="warning">
			<p>Running in browser mode</p>
			<p class="hint">Run with <code>npm run tauri dev</code> for full functionality</p>
		</div>
	{:else}
		<SoundGrid sounds={soundsState.sounds} />
	{/if}

	<NowPlaying />
</main>

<style>
	main {
		height: 100vh;
		display: flex;
		flex-direction: column;
		padding-bottom: 60px; /* Space for NowPlaying bar */
	}

	header {
		padding: 1rem 1.5rem;
		border-bottom: 1px solid var(--color-border);
		flex-shrink: 0;
	}

	h1 {
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--color-primary);
		margin-bottom: 0.25rem;
	}

	.subtitle {
		font-size: 0.9rem;
		color: var(--color-text-muted);
	}

	.loading,
	.error,
	.warning {
		padding: 2rem;
		text-align: center;
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
	}

	.error {
		color: var(--color-primary);
	}

	.warning {
		color: var(--color-text-muted);
	}

	.warning .hint {
		font-size: 0.85rem;
		opacity: 0.7;
	}

	.warning code {
		background: var(--color-bg-secondary);
		padding: 0.2rem 0.5rem;
		border-radius: var(--radius-sm);
		font-family: monospace;
	}
</style>
