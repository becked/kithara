<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		getExtractionStatus,
		startExtraction,
		cancelExtraction,
		detectGamePath
	} from '$lib/api';
	import type { ExtractionStatus } from '$lib/types';

	let { onComplete }: { onComplete?: () => void } = $props();

	let status = $state<ExtractionStatus>({
		state: 'not_started',
		progress: 0,
		currentFile: null,
		error: null
	});

	let gamePath = $state<string | null>(null);
	let pollInterval: ReturnType<typeof setInterval> | null = null;

	// Computed state helpers
	let isNotStarted = $derived(status.state === 'not_started');
	let isInProgress = $derived(status.state === 'in_progress');
	let isComplete = $derived(status.state === 'complete');
	let hasError = $derived(status.state === 'error');
	let progressPercent = $derived(Math.round(status.progress * 100));

	async function detectPath() {
		try {
			gamePath = await detectGamePath();
		} catch (e) {
			console.error('Failed to detect game path:', e);
		}
	}

	async function handleStart() {
		if (!gamePath) {
			return;
		}

		try {
			await startExtraction(gamePath);
			startPolling();
		} catch (e) {
			console.error('Failed to start extraction:', e);
			status = {
				...status,
				state: 'error',
				error: String(e)
			};
		}
	}

	async function handleCancel() {
		try {
			await cancelExtraction();
		} catch (e) {
			console.error('Failed to cancel extraction:', e);
		}
	}

	function startPolling() {
		if (pollInterval) return;

		pollInterval = setInterval(async () => {
			try {
				status = await getExtractionStatus();

				// Stop polling when complete or error
				if (status.state === 'complete') {
					stopPolling();
					onComplete?.();
				} else if (status.state === 'error') {
					stopPolling();
				}
			} catch (e) {
				console.error('Failed to get status:', e);
			}
		}, 250);
	}

	function stopPolling() {
		if (pollInterval) {
			clearInterval(pollInterval);
			pollInterval = null;
		}
	}

	onMount(async () => {
		await detectPath();
		status = await getExtractionStatus();

		// Resume polling if extraction was in progress
		if (status.state === 'in_progress') {
			startPolling();
		}
	});

	onDestroy(() => {
		stopPolling();
	});
</script>

<div class="extraction-container">
	{#if isNotStarted}
		<div class="extraction-setup">
			<h2>Extract Audio</h2>
			<p class="description">
				Extract sound effects from Old World's game files. This process takes 3-4 minutes.
			</p>

			{#if gamePath}
				<div class="game-path">
					<span class="label">Game found:</span>
					<span class="path">{gamePath}</span>
				</div>
			{:else}
				<div class="game-path not-found">
					<p>Old World installation not detected.</p>
					<p class="hint">Make sure Old World is installed via Steam.</p>
				</div>
			{/if}

			<button class="primary-button" onclick={handleStart} disabled={!gamePath}>
				Start Extraction
			</button>
		</div>
	{:else if isInProgress}
		<div class="extraction-progress">
			<h2>Extracting Audio...</h2>

			<div class="progress-bar-container">
				<div class="progress-bar" style="width: {progressPercent}%"></div>
			</div>

			<div class="progress-info">
				<span class="percent">{progressPercent}%</span>
				{#if status.currentFile}
					<span class="current-file">{status.currentFile}</span>
				{/if}
			</div>

			<button class="cancel-button" onclick={handleCancel}> Cancel </button>
		</div>
	{:else if isComplete}
		<div class="extraction-complete">
			<h2>Extraction Complete</h2>
			<p>Successfully extracted audio files.</p>
		</div>
	{:else if hasError}
		<div class="extraction-error">
			<h2>Extraction Failed</h2>
			<p class="error-message">{status.error}</p>
			<button class="primary-button" onclick={handleStart}> Retry </button>
		</div>
	{/if}
</div>

<style>
	.extraction-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		min-height: 400px;
		padding: 2rem;
		text-align: center;
	}

	h2 {
		font-size: 1.5rem;
		margin-bottom: 1rem;
		color: var(--color-text);
	}

	.description {
		color: var(--color-text-muted);
		margin-bottom: 1.5rem;
		max-width: 400px;
	}

	.game-path {
		font-size: 0.9rem;
		margin-bottom: 1.5rem;
		padding: 0.75rem 1rem;
		background: var(--color-bg-secondary);
		border-radius: var(--radius-md);
		max-width: 500px;
		word-break: break-all;
	}

	.game-path .label {
		color: var(--color-text-muted);
		display: block;
		margin-bottom: 0.25rem;
	}

	.game-path .path {
		color: var(--color-text);
		font-family: monospace;
		font-size: 0.8rem;
	}

	.game-path.not-found {
		color: var(--color-text-muted);
	}

	.game-path.not-found .hint {
		font-size: 0.85rem;
		opacity: 0.7;
		margin-top: 0.5rem;
	}

	.primary-button {
		padding: 0.75rem 2rem;
		background: var(--color-primary);
		color: white;
		border: none;
		border-radius: var(--radius-md);
		font-weight: 600;
		cursor: pointer;
		transition: background 0.2s;
	}

	.primary-button:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.primary-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.cancel-button {
		padding: 0.5rem 1.5rem;
		background: transparent;
		color: var(--color-text-muted);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		margin-top: 1rem;
		cursor: pointer;
	}

	.cancel-button:hover {
		color: var(--color-text);
		border-color: var(--color-text-muted);
	}

	/* Progress bar */
	.progress-bar-container {
		width: 100%;
		max-width: 400px;
		height: 8px;
		background: var(--color-bg-secondary);
		border-radius: var(--radius-sm);
		overflow: hidden;
		margin-bottom: 1rem;
	}

	.progress-bar {
		height: 100%;
		background: var(--color-primary);
		transition: width 0.2s ease-out;
	}

	.progress-info {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		color: var(--color-text-muted);
	}

	.percent {
		font-size: 1.25rem;
		font-weight: 600;
		color: var(--color-text);
	}

	.current-file {
		font-size: 0.85rem;
		font-family: monospace;
		max-width: 300px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.error-message {
		color: var(--color-primary);
		padding: 1rem;
		background: var(--color-bg-secondary);
		border-radius: var(--radius-md);
		margin-bottom: 1rem;
		font-family: monospace;
		font-size: 0.9rem;
		max-width: 400px;
		word-break: break-word;
	}

	.extraction-complete {
		color: var(--color-text);
	}

	.extraction-complete p {
		color: var(--color-text-muted);
	}
</style>
