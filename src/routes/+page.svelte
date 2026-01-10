<script lang="ts">
	import { onMount } from 'svelte';
	import SoundGrid from '$lib/components/SoundGrid.svelte';
	import NowPlaying from '$lib/components/NowPlaying.svelte';
	import Search from '$lib/components/Search.svelte';
	import CategorySidebar from '$lib/components/CategorySidebar.svelte';
	import UnitFilter from '$lib/components/UnitFilter.svelte';
	import ExtractionProgress from '$lib/components/ExtractionProgress.svelte';
	import { soundsState, filterState, initializeSounds, fetchSounds } from '$lib/stores/sounds.svelte';
	import { getExtractionStatus } from '$lib/api';

	let tauriAvailable = $state(false);
	let initialized = $state(false);
	let needsExtraction = $state(false);
	let checkingStatus = $state(true);

	// Re-fetch sounds when filters change (after initialization)
	$effect(() => {
		// Access all filter values to track them as dependencies
		const query = filterState.query;
		const category = filterState.category;
		const unitType = filterState.unitType;
		const showFavoritesOnly = filterState.showFavoritesOnly;

		if (tauriAvailable && initialized && !needsExtraction) {
			fetchSounds();
		}
	});

	async function handleExtractionComplete() {
		needsExtraction = false;
		await initializeSounds();
		initialized = true;
	}

	onMount(async () => {
		if ('__TAURI__' in window) {
			tauriAvailable = true;

			// Check extraction status first
			try {
				const status = await getExtractionStatus();

				// If extraction is in progress, show extraction UI
				if (status.state === 'in_progress') {
					needsExtraction = true;
					checkingStatus = false;
					return;
				}

				// Initialize sounds to check if we have any
				await initializeSounds();
				initialized = true;

				// If no real sounds (only test sounds or empty), prompt for extraction
				// Check if we have more than just test category
				const hasRealSounds = soundsState.categories.some(
					(c) => c.id !== 'test' && c.count > 0
				);
				needsExtraction = !hasRealSounds && soundsState.sounds.length <= 3;
			} catch (e) {
				console.error('Failed to check extraction status:', e);
			}

			checkingStatus = false;
		} else {
			checkingStatus = false;
		}
	});
</script>

{#if checkingStatus}
	<div class="loading-screen">
		<div class="status-message">Loading...</div>
	</div>
{:else if needsExtraction}
	<div class="extraction-screen">
		<ExtractionProgress onComplete={handleExtractionComplete} />
	</div>
{:else}
	<div class="app-layout">
		<!-- Sidebar -->
		{#if tauriAvailable}
			<CategorySidebar categories={soundsState.categories} favoritesCount={soundsState.favoritesCount} />
		{/if}

		<!-- Main content -->
		<main class="main-content">
			<header>
				<div class="header-top">
					<div class="title-area">
						<h1>Kithara</h1>
						<p class="subtitle">Old World Soundboard</p>
					</div>
				</div>
				{#if tauriAvailable}
					<div class="header-controls">
						<Search />
						<UnitFilter unitTypes={soundsState.unitTypes} />
					</div>
				{/if}
			</header>

			{#if soundsState.loading && !initialized}
				<div class="status-message">Loading sounds...</div>
			{:else if soundsState.error}
				<div class="status-message error">{soundsState.error}</div>
			{:else if !tauriAvailable}
				<div class="status-message">
					<p>Running in browser mode</p>
					<p class="hint">Run with <code>npm run tauri dev</code> for full functionality</p>
				</div>
			{:else if soundsState.sounds.length === 0}
				<div class="status-message">
					<p>No sounds found</p>
					{#if filterState.showFavoritesOnly}
						<p class="hint">No favorites yet. Click the heart icon on any sound to add it.</p>
					{:else if filterState.query || filterState.category || filterState.unitType}
						<p class="hint">Try adjusting your filters</p>
					{/if}
				</div>
			{:else}
				<SoundGrid sounds={soundsState.sounds} />
			{/if}
		</main>

		<NowPlaying />
	</div>
{/if}

<style>
	.loading-screen,
	.extraction-screen {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100vh;
		background: var(--color-bg);
	}

	.app-layout {
		display: flex;
		height: 100vh;
		padding-bottom: 60px; /* Space for NowPlaying bar */
	}

	.main-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-width: 0; /* Prevent flex item from overflowing */
	}

	header {
		padding: 1rem 1.5rem;
		border-bottom: 1px solid var(--color-border);
		flex-shrink: 0;
	}

	.header-top {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 1rem;
	}

	.title-area h1 {
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--color-primary);
		margin-bottom: 0.25rem;
	}

	.subtitle {
		font-size: 0.9rem;
		color: var(--color-text-muted);
	}

	.header-controls {
		display: flex;
		gap: 1rem;
		align-items: center;
	}

	/* Search takes remaining space */
	.header-controls :global(.search-container) {
		flex: 1;
		max-width: 400px;
	}

	.status-message {
		padding: 2rem;
		text-align: center;
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
		color: var(--color-text-muted);
	}

	.status-message.error {
		color: var(--color-primary);
	}

	.status-message .hint {
		font-size: 0.85rem;
		opacity: 0.7;
	}

	.status-message code {
		background: var(--color-bg-secondary);
		padding: 0.2rem 0.5rem;
		border-radius: var(--radius-sm);
		font-family: monospace;
	}
</style>
