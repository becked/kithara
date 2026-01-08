<script lang="ts">
	import { onMount } from 'svelte';

	let status = $state('Initializing...');
	let tauriAvailable = $state(false);

	onMount(async () => {
		// Check if we're running in Tauri
		if ('__TAURI__' in window) {
			tauriAvailable = true;
			status = 'Tauri backend connected';
		} else {
			status = 'Running in browser (Tauri not available)';
		}
	});
</script>

<main>
	<div class="hero">
		<h1>Kithara</h1>
		<p class="subtitle">Old World Soundboard</p>
	</div>

	<div class="status">
		<div class="status-indicator" class:connected={tauriAvailable}></div>
		<span>{status}</span>
	</div>

	<div class="placeholder">
		<p>Sound grid will appear here once extraction is complete.</p>
	</div>
</main>

<style>
	main {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 2rem;
		gap: 2rem;
	}

	.hero {
		text-align: center;
	}

	h1 {
		font-size: 3rem;
		font-weight: 700;
		color: var(--color-primary);
		margin-bottom: 0.5rem;
	}

	.subtitle {
		font-size: 1.2rem;
		color: var(--color-text-muted);
	}

	.status {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.75rem 1.5rem;
		background: var(--color-bg-secondary);
		border-radius: var(--radius-md);
		border: 1px solid var(--color-border);
	}

	.status-indicator {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		background: var(--color-text-muted);
	}

	.status-indicator.connected {
		background: #4ade80;
	}

	.placeholder {
		padding: 3rem;
		background: var(--color-bg-secondary);
		border-radius: var(--radius-lg);
		border: 2px dashed var(--color-border);
		color: var(--color-text-muted);
		text-align: center;
		max-width: 400px;
	}
</style>
