<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	let { isPlaying = false, isPaused = false }: { isPlaying?: boolean; isPaused?: boolean } =
		$props();

	const BAR_COUNT = 19;
	let bars = $state<number[]>(Array(BAR_COUNT).fill(0));
	let animationFrame: number;
	let lastTime = 0;

	// Simulate audio visualization with pseudo-random heights
	function animate(currentTime: number) {
		if (!lastTime) lastTime = currentTime;
		const delta = (currentTime - lastTime) / 1000;
		lastTime = currentTime;

		if (isPlaying && !isPaused) {
			// Update bars with pseudo-random movement
			bars = bars.map((height, i) => {
				// Each bar has its own "frequency" based on position
				const baseFreq = 2 + (i % 5) * 0.5;
				const phase = (currentTime / 1000) * baseFreq + i * 0.3;
				const noise = Math.sin(phase) * 0.3 + Math.sin(phase * 1.7) * 0.2;

				// Target height with some randomness
				const targetHeight = 0.3 + noise * 0.5 + Math.random() * 0.2;

				// Smooth interpolation
				const smoothFactor = delta * 15;
				return height + (targetHeight - height) * Math.min(smoothFactor, 1);
			});
		} else {
			// Decay when not playing
			bars = bars.map((height) => Math.max(0, height - 0.05));
		}

		animationFrame = requestAnimationFrame(animate);
	}

	onMount(() => {
		animationFrame = requestAnimationFrame(animate);
	});

	onDestroy(() => {
		if (animationFrame) {
			cancelAnimationFrame(animationFrame);
		}
	});
</script>

<div class="visualizer">
	{#each bars as height, i}
		<div class="bar-container">
			<div class="bar" style="height: {Math.max(2, height * 100)}%">
				<div class="bar-peak"></div>
			</div>
		</div>
	{/each}
</div>

<style>
	.visualizer {
		display: flex;
		align-items: flex-end;
		justify-content: space-between;
		gap: calc(1px * var(--winamp-scale, 2));
		width: 100%;
		height: 100%;
		background: #000;
		padding: calc(1px * var(--winamp-scale, 2));
	}

	.bar-container {
		flex: 1;
		height: 100%;
		display: flex;
		align-items: flex-end;
	}

	.bar {
		width: 100%;
		min-height: calc(1px * var(--winamp-scale, 2));
		background: linear-gradient(
			to top,
			#2a5a2a 0%,
			#3a8a3a 30%,
			#5aba5a 60%,
			#8ada8a 80%,
			#aafa7a 100%
		);
		position: relative;
		transition: height 0.05s linear;
		image-rendering: pixelated;
	}

	.bar-peak {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: calc(2px * var(--winamp-scale, 2));
		background: #aafa7a;
	}
</style>
