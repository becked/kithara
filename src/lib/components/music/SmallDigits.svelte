<script lang="ts">
	/**
	 * Renders small digits using TEXT.BMP sprite sheet.
	 * TEXT.BMP layout (this skin):
	 * - Each character is 5x6 pixels
	 * - 31 characters per row, 12 rows total
	 * - Row 0 (y=0): A-Z uppercase letters
	 * - Row 1 (y=6): numbers "0123456789" starting at position 0
	 */
	let { value = 0, padTo = 0 }: { value?: number; padTo?: number } = $props();

	// Convert number to string, optionally padded
	let digits = $derived(() => {
		const str = String(Math.floor(value));
		if (padTo > 0 && str.length < padTo) {
			return str.padStart(padTo, ' ');
		}
		return str;
	});

	// Get sprite position for a character
	function getCharPosition(char: string): { x: number; y: number } {
		const code = char.charCodeAt(0);

		// Numbers 0-9 are on row 1 (y=6), starting at x=0
		if (code >= 48 && code <= 57) {
			// '0' is 48, '9' is 57
			const digitIndex = code - 48;
			return { x: digitIndex * 5, y: 6 };
		}

		// Space - return position for blank
		if (char === ' ') {
			return { x: 30 * 5, y: 0 }; // Usually a blank space at end
		}

		// Default fallback
		return { x: 0, y: 12 };
	}
</script>

<div class="small-digits">
	{#each digits().split('') as char}
		{@const pos = getCharPosition(char)}
		<span
			class="small-char"
			class:space={char === ' '}
			style="--char-x: {pos.x}px; --char-y: {pos.y}px"
		></span>
	{/each}
</div>

<style>
	.small-digits {
		display: flex;
		gap: 0;
	}

	.small-char {
		width: calc(5px * var(--winamp-scale, 2));
		height: calc(6px * var(--winamp-scale, 2));
		background-image: url('/skins/winamp/TEXT.BMP');
		background-size: calc(155px * var(--winamp-scale, 2)) calc(74px * var(--winamp-scale, 2));
		background-position: calc(var(--char-x) * var(--winamp-scale, 2) * -1) calc(var(--char-y) * var(--winamp-scale, 2) * -1);
		background-repeat: no-repeat;
		image-rendering: pixelated;
	}

	.small-char.space {
		background: transparent;
	}
</style>
