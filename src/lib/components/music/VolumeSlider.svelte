<script lang="ts">
	let {
		volume = 0.8,
		onVolumeChange
	}: {
		volume?: number;
		onVolumeChange?: (volume: number) => void;
	} = $props();

	let volumePercent = $derived(volume * 100);

	function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		const newVolume = parseFloat(target.value) / 100;
		onVolumeChange?.(newVolume);
	}
</script>

<div class="volume-slider">
	<div class="volume-track">
		<div class="volume-fill" style="width: {volumePercent}%"></div>
		<input
			type="range"
			min="0"
			max="100"
			step="1"
			value={volumePercent}
			oninput={handleInput}
			class="volume-input"
		/>
	</div>
</div>

<style>
	.volume-slider {
		width: 100%;
		height: calc(14px * var(--winamp-scale, 2));
		display: flex;
		align-items: center;
	}

	.volume-track {
		position: relative;
		width: 100%;
		height: calc(6px * var(--winamp-scale, 2));
		background: #1a1a1a;
		border-radius: calc(2px * var(--winamp-scale, 2));
		overflow: visible;
	}

	.volume-fill {
		position: absolute;
		top: 0;
		left: 0;
		height: 100%;
		background: linear-gradient(to right, #3a5a3a, #6aba6a);
		border-radius: calc(2px * var(--winamp-scale, 2));
		pointer-events: none;
	}

	.volume-input {
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

	.volume-input::-webkit-slider-thumb {
		-webkit-appearance: none;
		width: calc(14px * var(--winamp-scale, 2));
		height: calc(11px * var(--winamp-scale, 2));
		background-image: url('/skins/winamp/VOLUME.BMP');
		background-size: calc(68px * var(--winamp-scale, 2)) calc(420px * var(--winamp-scale, 2));
		background-position: 0 calc(-422px * var(--winamp-scale, 2));
		background-repeat: no-repeat;
		image-rendering: pixelated;
		cursor: grab;
		border: none;
	}

	.volume-input::-webkit-slider-thumb:active {
		cursor: grabbing;
		background-position: calc(-15px * var(--winamp-scale, 2)) calc(-422px * var(--winamp-scale, 2));
	}

	.volume-input::-moz-range-thumb {
		width: calc(14px * var(--winamp-scale, 2));
		height: calc(11px * var(--winamp-scale, 2));
		background-image: url('/skins/winamp/VOLUME.BMP');
		background-size: calc(68px * var(--winamp-scale, 2)) calc(420px * var(--winamp-scale, 2));
		background-position: 0 calc(-422px * var(--winamp-scale, 2));
		background-repeat: no-repeat;
		image-rendering: pixelated;
		cursor: grab;
		border: none;
	}

	.volume-input::-moz-range-thumb:active {
		cursor: grabbing;
		background-position: calc(-15px * var(--winamp-scale, 2)) calc(-422px * var(--winamp-scale, 2));
	}

	.volume-input::-webkit-slider-runnable-track {
		height: calc(6px * var(--winamp-scale, 2));
		background: transparent;
	}

	.volume-input::-moz-range-track {
		height: calc(6px * var(--winamp-scale, 2));
		background: transparent;
	}
</style>
