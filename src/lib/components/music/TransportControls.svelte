<script lang="ts">
	let {
		isPlaying = false,
		isPaused = false,
		onPlay,
		onPause,
		onStop,
		onPrev,
		onNext
	}: {
		isPlaying?: boolean;
		isPaused?: boolean;
		onPlay?: () => void;
		onPause?: () => void;
		onStop?: () => void;
		onPrev?: () => void;
		onNext?: () => void;
	} = $props();

	function handlePlayPause() {
		if (isPlaying && !isPaused) {
			onPause?.();
		} else {
			onPlay?.();
		}
	}

	function handlePauseToggle() {
		if (isPaused) {
			onPlay?.();  // Resume
		} else if (isPlaying) {
			onPause?.();
		}
	}
</script>

<div class="transport-controls">
	<button class="transport-btn prev" onclick={onPrev} title="Previous">
		<span class="btn-icon"></span>
	</button>
	<button class="transport-btn play" onclick={handlePlayPause} title={isPlaying && !isPaused ? 'Pause' : 'Play'}>
		<span class="btn-icon" class:playing={isPlaying && !isPaused}></span>
	</button>
	<button class="transport-btn pause" onclick={handlePauseToggle} title={isPaused ? 'Resume' : 'Pause'}>
		<span class="btn-icon" class:active={isPaused}></span>
	</button>
	<button class="transport-btn stop" onclick={onStop} title="Stop">
		<span class="btn-icon"></span>
	</button>
	<button class="transport-btn next" onclick={onNext} title="Next">
		<span class="btn-icon"></span>
	</button>
</div>

<style>
	.transport-controls {
		display: flex;
		gap: 0;
	}

	.transport-btn {
		background: none;
		border: none;
		padding: 0;
		cursor: pointer;
		width: calc(23px * var(--winamp-scale, 2));
		height: calc(18px * var(--winamp-scale, 2));
		background-image: url('/skins/winamp/CBUTTONS.BMP');
		background-size: calc(137px * var(--winamp-scale, 2)) calc(36px * var(--winamp-scale, 2));
		background-repeat: no-repeat;
		image-rendering: pixelated;
		transition: filter 0.1s;
	}

	.transport-btn .btn-icon {
		display: none;
	}

	.transport-btn:hover {
		filter: brightness(1.2);
	}

	.transport-btn:active {
		filter: brightness(0.9);
	}

	/* Previous button - first in sprite row */
	.prev {
		background-position: 0 0;
	}
	.prev:active {
		background-position: 0 calc(-18px * var(--winamp-scale, 2));
	}

	/* Play button - second in sprite */
	.play {
		background-position: calc(-23px * var(--winamp-scale, 2)) 0;
	}
	.play:active {
		background-position: calc(-23px * var(--winamp-scale, 2)) calc(-18px * var(--winamp-scale, 2));
	}

	/* Pause button - third in sprite */
	.pause {
		background-position: calc(-46px * var(--winamp-scale, 2)) 0;
	}
	.pause:active,
	.pause .btn-icon.active {
		background-position: calc(-46px * var(--winamp-scale, 2)) calc(-18px * var(--winamp-scale, 2));
	}

	/* Stop button - fourth in sprite */
	.stop {
		background-position: calc(-69px * var(--winamp-scale, 2)) 0;
	}
	.stop:active {
		background-position: calc(-69px * var(--winamp-scale, 2)) calc(-18px * var(--winamp-scale, 2));
	}

	/* Next button - fifth in sprite */
	.next {
		width: calc(22px * var(--winamp-scale, 2));
		background-position: calc(-92px * var(--winamp-scale, 2)) 0;
	}
	.next:active {
		background-position: calc(-92px * var(--winamp-scale, 2)) calc(-18px * var(--winamp-scale, 2));
	}
</style>
