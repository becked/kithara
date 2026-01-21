<script lang="ts">
	let { isPlaying = false, isPaused = false }: { isPlaying?: boolean; isPaused?: boolean } =
		$props();

	// Determine which indicator to show
	let state = $derived(isPaused ? 'paused' : isPlaying ? 'playing' : 'stopped');
</script>

<div class="play-indicator" class:playing={state === 'playing'} class:paused={state === 'paused'}>
	{#if state === 'playing'}
		<div class="indicator-icon play-icon">
			<div class="play-arrow"></div>
		</div>
	{:else if state === 'paused'}
		<div class="indicator-icon pause-icon">
			<div class="pause-bar"></div>
			<div class="pause-bar"></div>
		</div>
	{:else}
		<div class="indicator-icon stop-icon">
			<div class="stop-square"></div>
		</div>
	{/if}
</div>

<style>
	.play-indicator {
		width: calc(9px * var(--winamp-scale, 2));
		height: calc(9px * var(--winamp-scale, 2));
		background-image: url('/skins/winamp/PLAYPAUS.BMP');
		background-size: calc(36px * var(--winamp-scale, 2)) calc(9px * var(--winamp-scale, 2));
		background-repeat: no-repeat;
		image-rendering: pixelated;
		/* Default: stopped state */
		background-position: calc(-18px * var(--winamp-scale, 2)) 0;
	}

	.play-indicator.playing {
		background-position: 0 0;
	}

	.play-indicator.paused {
		background-position: calc(-9px * var(--winamp-scale, 2)) 0;
	}

	.indicator-icon {
		display: none;
	}

	/* Fallback CSS-based indicators if sprite doesn't work */
	.play-icon {
		display: none;
	}

	.pause-icon {
		display: none;
	}

	.stop-icon {
		display: none;
	}

	.play-arrow {
		width: 0;
		height: 0;
		border-top: calc(4px * var(--winamp-scale, 2)) solid transparent;
		border-bottom: calc(4px * var(--winamp-scale, 2)) solid transparent;
		border-left: calc(6px * var(--winamp-scale, 2)) solid #00cc00;
	}

	.pause-bar {
		width: calc(2px * var(--winamp-scale, 2));
		height: calc(8px * var(--winamp-scale, 2));
		background: #00cc00;
	}

	.pause-icon {
		gap: calc(2px * var(--winamp-scale, 2));
	}

	.stop-square {
		width: calc(6px * var(--winamp-scale, 2));
		height: calc(6px * var(--winamp-scale, 2));
		background: #00cc00;
	}
</style>
