<script lang="ts">
	import type { Sound } from '$lib/api';
	import SoundButton from './SoundButton.svelte';

	let { sounds }: { sounds: Sound[] } = $props();
</script>

{#if sounds.length > 0}
	<div class="sound-grid">
		{#each sounds as sound (sound.id)}
			<SoundButton {sound} />
		{/each}
	</div>
{:else}
	<div class="empty-state">
		<p>No sounds to display</p>
	</div>
{/if}

<style>
	.sound-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
		gap: 0.75rem;
		padding: 1rem;
		overflow-y: auto;
		flex: 1;
	}

	.empty-state {
		text-align: center;
		padding: 3rem;
		color: var(--color-text-muted);
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
	}
</style>
