<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	let { text = '' }: { text?: string } = $props();

	let offset = $state(0);
	let tickerRef: HTMLDivElement;
	let animationFrame: number;
	let lastTime = 0;
	const scrollSpeed = 30; // pixels per second

	// Add padding for scrolling effect
	let displayText = $derived(text ? `${text}  ***  ` : '');

	function animate(currentTime: number) {
		if (!lastTime) lastTime = currentTime;
		const delta = (currentTime - lastTime) / 1000;
		lastTime = currentTime;

		if (tickerRef) {
			const textWidth = tickerRef.scrollWidth;
			const containerWidth = tickerRef.clientWidth;

			if (textWidth > containerWidth) {
				offset += delta * scrollSpeed;
				if (offset > textWidth / 2) {
					offset = 0;
				}
			} else {
				offset = 0;
			}
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

<div class="ticker-container">
	<div class="ticker-text" bind:this={tickerRef} style="transform: translateX(-{offset}px)">
		<span>{displayText}</span>
		<span>{displayText}</span>
	</div>
</div>

<style>
	.ticker-container {
		width: 100%;
		height: 100%;
		overflow: hidden;
		background: transparent;
	}

	.ticker-text {
		display: flex;
		white-space: nowrap;
		font-size: calc(7px * var(--winamp-scale, 2));
		color: #00cc00;
		text-transform: uppercase;
		letter-spacing: calc(1px * var(--winamp-scale, 2));
		font-family: 'Courier New', monospace;
		text-shadow: 0 0 calc(1px * var(--winamp-scale, 2)) #00ff00;
		line-height: 1;
	}

	.ticker-text span {
		flex-shrink: 0;
	}
</style>
