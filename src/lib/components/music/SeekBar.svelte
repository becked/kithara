<script lang="ts">
	let {
		position = 0,
		duration = 0,
		onSeek
	}: {
		position?: number;
		duration?: number;
		onSeek?: (position: number) => void;
	} = $props();

	let isDragging = $state(false);
	let dragProgress = $state(0);

	// Show drag position while dragging, otherwise show actual position (clamped to 0-100)
	let displayProgress = $derived(
		isDragging
			? dragProgress
			: duration > 0
				? Math.min(100, Math.max(0, (position / duration) * 100))
				: 0
	);

	function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		dragProgress = parseFloat(target.value);
	}

	function handleMouseDown(e: Event) {
		isDragging = true;
		// Initialize drag position from current slider value
		const target = e.target as HTMLInputElement;
		dragProgress = parseFloat(target.value);
	}

	function handleMouseUp() {
		if (isDragging && duration > 0) {
			// Only seek when releasing the slider
			const newPosition = (dragProgress / 100) * duration;
			onSeek?.(newPosition);
		}
		isDragging = false;
	}
</script>

<div class="seek-bar">
	<div class="seek-track">
		<div class="seek-progress" style="width: {displayProgress}%"></div>
		<input
			type="range"
			min="0"
			max="100"
			step="0.1"
			value={displayProgress}
			oninput={handleInput}
			onmousedown={handleMouseDown}
			onmouseup={handleMouseUp}
			class="seek-input"
			class:dragging={isDragging}
		/>
	</div>
</div>

<style>
	.seek-bar {
		width: 100%;
		height: calc(10px * var(--winamp-scale, 2));
		display: flex;
		align-items: center;
	}

	.seek-track {
		position: relative;
		width: 100%;
		height: calc(6px * var(--winamp-scale, 2));
		background: #1a1a1a;
		border-radius: calc(2px * var(--winamp-scale, 2));
		overflow: visible;
	}

	.seek-progress {
		position: absolute;
		top: 0;
		left: 0;
		height: 100%;
		background: linear-gradient(to right, #3a6a3a, #5a9a5a);
		border-radius: calc(2px * var(--winamp-scale, 2));
		pointer-events: none;
	}

	.seek-input {
		position: absolute;
		top: 50%;
		left: 0;
		width: 100%;
		height: calc(14px * var(--winamp-scale, 2));
		transform: translateY(-50%);
		-webkit-appearance: none;
		appearance: none;
		background: transparent;
		cursor: pointer;
		margin: 0;
	}

	.seek-input::-webkit-slider-thumb {
		-webkit-appearance: none;
		width: calc(28px * var(--winamp-scale, 2));
		height: calc(10px * var(--winamp-scale, 2));
		background-image: url('/skins/winamp/POSBAR.BMP');
		background-size: calc(307px * var(--winamp-scale, 2)) calc(10px * var(--winamp-scale, 2));
		background-position: calc(-248px * var(--winamp-scale, 2)) 0;
		background-repeat: no-repeat;
		image-rendering: pixelated;
		cursor: grab;
		border: none;
	}

	.seek-input::-webkit-slider-thumb:active,
	.seek-input.dragging::-webkit-slider-thumb {
		cursor: grabbing;
		background-position: calc(-278px * var(--winamp-scale, 2)) 0;
	}

	.seek-input::-moz-range-thumb {
		width: calc(28px * var(--winamp-scale, 2));
		height: calc(10px * var(--winamp-scale, 2));
		background-image: url('/skins/winamp/POSBAR.BMP');
		background-size: calc(307px * var(--winamp-scale, 2)) calc(10px * var(--winamp-scale, 2));
		background-position: calc(-248px * var(--winamp-scale, 2)) 0;
		background-repeat: no-repeat;
		image-rendering: pixelated;
		cursor: grab;
		border: none;
	}

	.seek-input::-moz-range-thumb:active {
		cursor: grabbing;
		background-position: calc(-278px * var(--winamp-scale, 2)) 0;
	}

	.seek-input::-webkit-slider-runnable-track {
		height: calc(6px * var(--winamp-scale, 2));
		background: transparent;
	}

	.seek-input::-moz-range-track {
		height: calc(6px * var(--winamp-scale, 2));
		background: transparent;
	}
</style>
