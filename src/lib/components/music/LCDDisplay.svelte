<script lang="ts">
	let { minutes = 0, seconds = 0 }: { minutes?: number; seconds?: number } = $props();

	// Split into individual digits
	let minTens = $derived(Math.floor(minutes / 10) % 10);
	let minOnes = $derived(minutes % 10);
	let secTens = $derived(Math.floor(seconds / 10) % 10);
	let secOnes = $derived(seconds % 10);
</script>

<div class="lcd-display">
	<span class="digit" style="--digit: {minTens}"></span>
	<span class="digit" style="--digit: {minOnes}"></span>
	<span class="colon"></span>
	<span class="digit" style="--digit: {secTens}"></span>
	<span class="digit" style="--digit: {secOnes}"></span>
</div>

<style>
	.lcd-display {
		display: flex;
		align-items: center;
		gap: calc(1px * var(--winamp-scale, 2));
	}

	.digit {
		width: calc(9px * var(--winamp-scale, 2));
		height: calc(13px * var(--winamp-scale, 2));
		background-image: url('/skins/winamp/NUMBERS.BMP');
		background-size: calc(99px * var(--winamp-scale, 2)) calc(13px * var(--winamp-scale, 2));
		background-position: calc(var(--digit) * -9px * var(--winamp-scale, 2)) 0;
		background-repeat: no-repeat;
		image-rendering: pixelated;
	}

	.colon {
		width: calc(3px * var(--winamp-scale, 2));
		height: calc(13px * var(--winamp-scale, 2));
		display: flex;
		flex-direction: column;
		justify-content: space-around;
		align-items: center;
		padding: calc(2px * var(--winamp-scale, 2)) 0;
	}

	.colon::before,
	.colon::after {
		content: '';
		width: calc(2px * var(--winamp-scale, 2));
		height: calc(2px * var(--winamp-scale, 2));
		background: #00cc00;
		border-radius: 50%;
	}
</style>
